use std::collections::BTreeMap;

use futures::{FutureExt, StreamExt};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::DefaultTerminal;

use tmux::{self, Preset, Session};

use crate::app::menus::Menu;
use crate::app::menus::create::CreateMenu;
use crate::app::menus::delete::DeleteMenu;
use crate::app::menus::presets::PresetsMenu;
use crate::app::menus::rename::RenameMenu;
use crate::app::menus::sessions::SessionsMenu;

#[derive(Debug, Clone, Default)]
pub enum Mode {
    #[default]
    Sessions,
    Presets,
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
    pub presets: BTreeMap<String, Preset>,
    pub presets_path: String,
    pub selected_session: Option<usize>,
    pub selected_preset: Option<usize>,
    pub exit: bool,
    pub mode: Mode,
}

#[derive(Clone, Debug)]
pub enum AppEvent {
    Error,
    Key(KeyEvent),
    Redraw,
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
        let (tx, rx) = mpsc::unbounded_channel();
        let _tx = tx.clone();

        let task = tokio::spawn(async move {
            let mut reader = crossterm::event::EventStream::new();
            loop {
                let crossterm_event = reader.next().fuse();
                tokio::select! {
                    Some(Ok(evt)) = crossterm_event => {
                        match evt {
                            crossterm::event::Event::Key(key) => {
                                if key.kind == crossterm::event::KeyEventKind::Press {
                                    tx.send(AppEvent::Key(key)).unwrap();
                                }
                            },
                            crossterm::event::Event::Resize(_, _) | crossterm::event::Event::FocusGained => {
                                tx.send(AppEvent::Redraw).unwrap();
                            },
                            _ => {},
                        }
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
    pub fn new(
        sessions: Vec<Session>,
        presets: BTreeMap<String, Preset>,
        presets_file: String,
    ) -> Self {
        Self {
            state: AppState {
                mode: Mode::Sessions,
                exit: false,
                sessions,
                selected_session: None,
                presets,
                presets_path: presets_file,
                selected_preset: None,
                event_handler: EventHandler::new(),
            },
        }
    }

    /// runs the application's main loop until the user quits
    pub async fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<(), String> {
        let active_index = self.state.sessions.iter().position(|s| s.active);
        self.state.selected_session = active_index;
        self.state.selected_preset = if self.state.presets.is_empty() {
            None
        } else {
            Some(0)
        };

        let mut create_menu = CreateMenu::default();
        let mut rename_menu = RenameMenu::default();
        let mut delete_menu = DeleteMenu::default();
        let mut sessions_menu = SessionsMenu::new(active_index);
        let mut presets_menu = PresetsMenu::new(active_index);

        while !self.state.exit {
            // Draw phase
            terminal
                .draw(|frame| {
                    let area = frame.area();

                    // unconditionally render sessions menu
                    frame.render_stateful_widget(&mut sessions_menu, area, &mut self.state);

                    match self.state.mode {
                        Mode::Create => {
                            frame.render_stateful_widget(&mut create_menu, area, &mut self.state)
                        }
                        Mode::Rename => {
                            frame.render_stateful_widget(&mut rename_menu, area, &mut self.state)
                        }
                        Mode::Delete => {
                            frame.render_stateful_widget(&mut delete_menu, area, &mut self.state)
                        }
                        Mode::Sessions => {} // Nothing extra to draw
                        Mode::Presets => {
                            frame.render_stateful_widget(&mut presets_menu, area, &mut self.state)
                        }
                    }
                })
                .map_err(|_| "Terminal rendering error".to_string())?;

            // Get next event
            let event = self
                .state
                .event_handler
                .next()
                .await
                .map_err(|_| "Error with event handler!".to_string())?;

            if matches!(event, AppEvent::Key(KeyEvent { modifiers, code, .. })
                if modifiers == KeyModifiers::CONTROL
                && code == KeyCode::Char('c'))
            {
                self.state.exit = true;
            }

            // Handle said event
            // TODO: This looks stupid
            match self.state.mode {
                Mode::Sessions => sessions_menu.handle_event(event, &mut self.state),
                Mode::Create => create_menu.handle_event(event, &mut self.state),
                Mode::Rename => rename_menu.handle_event(event, &mut self.state),
                Mode::Delete => delete_menu.handle_event(event, &mut self.state),
                Mode::Presets => presets_menu.handle_event(event, &mut self.state),
            }

            // Refresh tmux sessions on each keystroke
            self.state.sessions = tmux::list_sessions()?;

            // TODO: This hurts the time complexity part of my brain. Fix it?
            for preset in self.state.presets.values_mut() {
                preset.running = false;
            }

            // Required to update which presets are running and which are dead
            // Fortunately, this uses a BTreeMap now so it's not as bad as a regular Vec<Preset>
            for session in self.state.sessions.iter() {
                if let Some(v) = self.state.presets.get_mut(&session.name) {
                    v.running = true;
                }
            }
        }

        Ok(())
    }
}
