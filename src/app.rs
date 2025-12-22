use std::io;

use tui_textarea::TextArea;

use ratatui::crossterm::event::{self as crossterm_event, Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::widgets::ListState;
use ratatui::{DefaultTerminal, Frame};

use crate::render::centered_fixed_rect;
use crate::{Global, State, tmux};

#[derive(Debug, Default)]
pub enum Mode {
    #[default]
    Main,
    Create,
    Rename,
    Delete,
}

#[derive(Default)]
pub struct App<'a> {
    pub exit: bool,
    pub session_list_state: ListState,
    pub sessions: Vec<Session>,
    pub mode: Mode,
    pub text_area: TextArea<'a>,
}

#[derive(Debug, Clone)]
pub struct Session {
    pub name: String,
    pub windows: String,
    pub active: bool,
}

impl<'a> App<'a> {
    pub fn new() -> Self {
        Self {
            sessions: tmux::list_sessions().unwrap(),
            ..Default::default()
        }
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

pub fn handle_events(event: &Event, state: &mut State, ctx: &mut Global) -> io::Result<()> {
    match event {
        Event::Key(key_event)
            if key_event.modifiers == KeyModifiers::CONTROL
                && key_event.code == KeyCode::Char('c') =>
        {
            self.exit();
        }
        Event::Key(key_event) => handle_key_event(*key_event),
        _ => {}
    };
    Ok(())
}

fn handle_key_event(key_event: KeyEvent) {
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
            _ => _ = self.text_area.input(key_event),
        },
        Mode::Rename => match key_event.code {
            KeyCode::Esc => {
                self.text_area = TextArea::default();
                self.mode = Mode::Main;
            }
            KeyCode::Enter => {
                if let Some(index) = self.session_list_state.selected() {
                    tmux::rename_session(
                        &self.sessions[index].name,
                        &self.text_area.lines().join(""),
                    )
                    .unwrap()
                };
                self.text_area = TextArea::default();
                self.mode = Mode::Main;
            }
            _ => _ = self.text_area.input(key_event),
        },
        Mode::Delete => match key_event.code {
            KeyCode::Char('y') | KeyCode::Enter => {
                if let Some(index) = self.session_list_state.selected() {
                    tmux::delete_session(&self.sessions[index].name).unwrap()
                };
                self.mode = Mode::Main;
            }
            KeyCode::Char('n') | KeyCode::Esc => self.mode = Mode::Main,
            _ => {}
        },
    };
}
