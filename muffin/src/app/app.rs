use std::io;
use std::time::Duration;

use futures::{FutureExt, StreamExt};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tui_textarea::TextArea;

use crossterm::event::KeyEvent;
use ratatui::widgets::ListState;
use ratatui::{DefaultTerminal, Frame};

use tmux_helper::{self, Session};

use crate::app::menus::create::CreateMenu;
use crate::app::menus::delete::DeleteMenu;
use crate::app::menus::rename::RenameMenu;
use crate::app::menus::sessions::SessionsMenu;
use crate::app::menus::traits::Menu;

#[derive(Debug, Clone, Default)]
pub enum Mode {
    #[default]
    Main,
    Create,
    Rename,
    Delete,
}

pub struct App {
    pub state: AppState,
}

pub struct AppState {
    pub event_handler: EventHandler,
    pub sessions: Vec<Session>,
    pub selected_session: Option<usize>,
    pub exit: bool,
    pub mode: Mode,
}

#[derive(Clone, Debug)]
pub enum AppEvent {
    Error,
    Tick,
    Key(KeyEvent),
    ShowNotification(String),
    ClearNotification,
}

#[derive(Debug)]
pub struct EventHandler {
    pub tx: mpsc::UnboundedSender<AppEvent>,
    rx: mpsc::UnboundedReceiver<AppEvent>,
    _task: Option<JoinHandle<()>>,
}

impl EventHandler {
    pub fn new() -> Self {
        let tick_rate = std::time::Duration::from_millis(33);

        let (tx, rx) = mpsc::unbounded_channel();
        let _tx = tx.clone();

        let task = tokio::spawn(async move {
            let mut reader = crossterm::event::EventStream::new();
            let mut interval = tokio::time::interval(tick_rate);
            loop {
                let delay = interval.tick();
                let crossterm_event = reader.next().fuse();
                tokio::select! {
                    Some(Ok(evt)) = crossterm_event => {
                        match evt {
                            crossterm::event::Event::Key(key) => {
                                if key.kind == crossterm::event::KeyEventKind::Press {
                                    tx.send(AppEvent::Key(key)).unwrap();
                                }
                            },
                            _ => {},
                        }
                    },
                    _ = delay => {
                        tx.send(AppEvent::Tick).unwrap();
                    },
                }
            }
        });

        Self {
            tx: _tx,
            rx,
            _task: Some(task),
        }
    }

    pub async fn next(&mut self) -> Result<AppEvent, ()> {
        self.rx.recv().await.ok_or(())
    }
}

impl App {
    pub fn new() -> Self {
        Self {
            state: AppState {
                mode: Mode::Main,
                exit: false,
                sessions: Vec::new(),
                selected_session: None,
                event_handler: EventHandler::new(),
            },
        }
    }

    /// runs the application's main loop until the user quits
    pub async fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<(), ()> {
        self.state.sessions = tmux_helper::list_sessions().unwrap();
        let active_index = self.state.sessions.iter().position(|s| s.active);
        self.state.selected_session = active_index;

        let mut create_menu = CreateMenu::default();
        let mut rename_menu = RenameMenu::default();
        let mut delete_menu = DeleteMenu::default();
        let mut sessions_menu = SessionsMenu::default();

        while !self.state.exit {
            let event = self.state.event_handler.next().await?;
            let should_reload_tmux = !matches!(event, AppEvent::Tick);
            match self.state.mode {
                Mode::Main => sessions_menu.handle_event(event, &mut self.state, terminal),
                Mode::Create => create_menu.handle_event(event, &mut self.state, terminal),
                Mode::Rename => rename_menu.handle_event(event, &mut self.state, terminal),
                Mode::Delete => delete_menu.handle_event(event, &mut self.state, terminal),
            }

            if should_reload_tmux  {
                self.state.sessions = tmux_helper::list_sessions().unwrap();
            }
        }

        Ok(())
    }

    // fn draw(&mut self, frame: &mut Frame) {
    //     frame.render_widget(self, frame.area());
    // }
}
