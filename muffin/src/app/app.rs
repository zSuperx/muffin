use std::io;
use std::time::Duration;

use futures::{FutureExt, StreamExt};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tui_textarea::TextArea;

use crossterm::event::{KeyEvent};
use ratatui::widgets::ListState;
use ratatui::{DefaultTerminal, Frame};

use tmux_helper::{self, Session};

#[derive(Debug, Clone, Default)]
pub enum Mode {
    #[default]
    Main,
    Presets,
    Create,
    Rename,
    Delete,
}

pub struct App<'a> {
    pub exit: bool,
    pub session_list_state: ListState,
    pub sessions: Vec<Session>,
    pub presets: Vec<Session>,
    pub mode: Mode,
    pub text_area: TextArea<'a>,
    pub event_handler: EventHandler,
    pub notification: Option<(Mode, String)>,
}

impl<'a> Default for App<'a> {
    fn default() -> Self {
        Self {
            exit: false,
            session_list_state: Default::default(),
            sessions: Vec::new(),
            presets: Vec::new(),
            mode: Mode::Main,
            text_area: Default::default(),
            event_handler: EventHandler::new(),
            notification: None,
        }
    }
}

#[derive(Clone, Debug)]
pub enum AppEvent {
    Error,
    Tick,
    Key(KeyEvent),
    ShowNotification((Mode, String)),
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

impl<'a> App<'a> {
    /// runs the application's main loop until the user quits
    pub async fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        self.sessions = tmux_helper::list_sessions().unwrap();
        let active_index = self.sessions.iter().position(|s| s.active);
        self.session_list_state.select(active_index);

        let mut should_reload_tmux;

        while !self.exit {
            should_reload_tmux = true;
            let event = self.event_handler.next().await.unwrap();
            match event {
                AppEvent::Key(key_event) => self.handle_key(key_event),
                AppEvent::ShowNotification(x) => self.notification = Some(x),
                AppEvent::ClearNotification => self.notification = None,
                AppEvent::Tick => {
                    should_reload_tmux = false;
                    terminal.draw(|frame| self.draw(frame))?;
                }
                _ => {}
            };

            // Reload tmux session list on all non-Tick events
            if should_reload_tmux {
                self.sessions = tmux_helper::list_sessions().unwrap()
            }
        }

        Ok(())
    }

    pub fn send_timed_notification(&mut self, mode: Mode, msg: String) {
        let tx = self.event_handler.tx.clone();

        // Immediately show notification
        let _ = tx.send(AppEvent::ShowNotification((mode, msg)));

        // Spawn a background task to clear it after 3 seconds
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_secs(3)).await;
            let _ = tx.send(AppEvent::ClearNotification);
        });
    }

    fn draw(&mut self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    pub fn select_next(&mut self) {
        self.session_list_state.select_next();
    }

    pub fn select_previous(&mut self) {
        self.session_list_state.select_previous();
    }

    pub fn select_first(&mut self) {
        self.session_list_state.select_first();
    }

    pub fn select_middle(&mut self) {
        let length = self.sessions.len();
        if length > 0 {
            let new_index = (length - 1).div_ceil(2);
            self.session_list_state.select(Some(new_index));
        }
    }

    pub fn select_last(&mut self) {
        self.session_list_state.select_last();
    }

    pub fn exit(&mut self) {
        self.exit = true;
    }
}
