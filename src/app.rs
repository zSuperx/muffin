use std::io;
use std::time::Duration;

use anyhow::Error;
use futures::{FutureExt, SinkExt, StreamExt};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tui_textarea::TextArea;

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::widgets::ListState;
use ratatui::{DefaultTerminal, Frame};

use crate::render::centered_fixed_rect;
use crate::tmux;

#[derive(Debug, Default)]
pub enum Mode {
    #[default]
    Main,
    Create,
    Rename,
    Delete,
}

pub struct App<'a> {
    pub exit: bool,
    pub session_list_state: ListState,
    pub sessions: Vec<Session>,
    pub mode: Mode,
    pub text_area: TextArea<'a>,
    pub event_handler: EventHandler,
    pub notification: Option<String>,
}

impl<'a> Default for App<'a> {
    fn default() -> Self {
        Self {
            exit: false,
            session_list_state: Default::default(),
            sessions: Vec::new(),
            mode: Mode::Main,
            text_area: Default::default(),
            event_handler: EventHandler::new(),
            notification: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Session {
    pub name: String,
    pub windows: String,
    pub active: bool,
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
    tx: mpsc::UnboundedSender<AppEvent>,
    rx: mpsc::UnboundedReceiver<AppEvent>,
    task: Option<JoinHandle<()>>,
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
            task: Some(task),
        }
    }

    pub async fn next(&mut self) -> Result<AppEvent, ()> {
        self.rx.recv().await.ok_or(())
    }
}

impl<'a> App<'a> {
    /// runs the application's main loop until the user quits
    pub async fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        self.sessions = tmux::list_sessions().unwrap();
        let active_index = self.sessions.iter().position(|s| s.active);
        self.session_list_state.select(active_index);

        let mut should_reload_tmux;

        while !self.exit {
            should_reload_tmux = true;
            let event = self.event_handler.next().await.unwrap();
            match event {
                AppEvent::Key(key_event) => self.handle_key_event(key_event),
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
                self.sessions = tmux::list_sessions().unwrap()
            }
        }

        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        frame.render_widget(self, centered_fixed_rect(frame.area(), 50, 20));
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        if key_event.modifiers == KeyModifiers::CONTROL && key_event.code == KeyCode::Char('c') {
            self.exit();
            return;
        }
        match self.mode {
            Mode::Main => {
                self.sessions = tmux::list_sessions().unwrap();
                match key_event.code {
                    KeyCode::Char('q') => self.exit(),
                    KeyCode::Down | KeyCode::Char('j') => self.select_next(),
                    KeyCode::Up | KeyCode::Char('k') => self.select_previous(),
                    KeyCode::Char('g') => self.select_first(),
                    KeyCode::Char('G') => self.select_last(),
                    KeyCode::Char('a') => self.mode = Mode::Create,
                    KeyCode::Char('r') => self.mode = Mode::Rename,
                    KeyCode::Char('d') => self.mode = Mode::Delete,
                    KeyCode::Enter => {
                        if let Some(index) = self.session_list_state.selected() {
                            tmux::switch_session(&self.sessions[index].name).unwrap()
                        };
                    }
                    _ => {}
                }
            }
            Mode::Create => match key_event.code {
                KeyCode::Esc => {
                    self.text_area = TextArea::default();
                    self.mode = Mode::Main;
                }
                KeyCode::Enter => {
                    tmux::create_session(&self.text_area.lines().join("\n")).unwrap();
                    self.text_area = TextArea::default();
                    self.mode = Mode::Main
                }
                // _ => _ = self.text_area.input(key_event),
                _ => {}
            },
            Mode::Rename => match key_event.code {
                KeyCode::Esc => {
                    self.text_area = TextArea::default();
                    self.mode = Mode::Main;
                }
                KeyCode::Enter => {
                    if let Some(index) = self.session_list_state.selected() {
                        match tmux::rename_session(
                            &self.sessions[index].name,
                            &self.text_area.lines().join(""),
                        ) {
                            Ok(_) => {
                                self.text_area = TextArea::default();
                                self.mode = Mode::Main;
                            }
                            Err(s) => trigger_timed_notification(self.event_handler.tx.clone(), s),
                        }
                    };
                }
                // _ => _ = self.text_area.input(key_event),
                _ => {}
            },
            Mode::Delete => match key_event.code {
                KeyCode::Char('y') | KeyCode::Enter => {
                    if let Some(index) = self.session_list_state.selected() {
                        match tmux::delete_session(&self.sessions[index].name) {
                            Ok(_) => {
                                self.text_area = TextArea::default();
                                self.mode = Mode::Main;
                            }
                            Err(s) => trigger_timed_notification(self.event_handler.tx.clone(), s),
                        }
                    };
                }
                KeyCode::Char('n') | KeyCode::Esc => self.mode = Mode::Main,
                _ => {}
            },
        };
    }

    fn select_next(&mut self) {
        self.session_list_state.select_next();
    }

    fn select_previous(&mut self) {
        self.session_list_state.select_previous();
    }

    fn select_first(&mut self) {
        self.session_list_state.select_first();
    }

    fn select_last(&mut self) {
        self.session_list_state.select_last();
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}
fn trigger_timed_notification(handler_tx: mpsc::UnboundedSender<AppEvent>, msg: String) {
    let tx = handler_tx.clone();

    // Immediately show notification
    let _ = tx.send(AppEvent::ShowNotification(msg));

    // Spawn a background task to clear it after 3 seconds
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(3)).await;
        let _ = tx.send(AppEvent::ClearNotification);
    });
}
