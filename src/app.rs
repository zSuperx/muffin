use anyhow::Error;
use rat_salsa::event::RenderedEvent;
use rat_salsa::timer::TimeOut;
use std::io;

use rat_salsa::{Control, SalsaAppContext, SalsaContext};
use tui_textarea::TextArea;

use ratatui::crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::widgets::ListState;

use crate::tmux;

#[derive(Debug, Default)]
pub enum Mode {
    #[default]
    Main,
    Create,
    Rename,
    Delete,
}

/// Application wide messages.
#[derive(Debug)]
pub enum AppEvent {
    Timer(TimeOut),
    Event(ratatui::crossterm::event::Event),
    Rendered,
    Message(String),
    Status(usize, String),
    AsyncMsg(String),
    AsyncTick(u32),
}
impl From<RenderedEvent> for AppEvent {
    fn from(_: RenderedEvent) -> Self {
        Self::Rendered
    }
}

impl From<TimeOut> for AppEvent {
    fn from(value: TimeOut) -> Self {
        Self::Timer(value)
    }
}

impl From<ratatui::crossterm::event::Event> for AppEvent {
    fn from(value: ratatui::crossterm::event::Event) -> Self {
        Self::Event(value)
    }
}
pub struct Config {}
pub struct Global {
    ctx: SalsaAppContext<AppEvent, Error>,

    pub cfg: Config,
}

impl SalsaContext<AppEvent, Error> for Global {
    fn set_salsa_ctx(&mut self, app_ctx: SalsaAppContext<AppEvent, Error>) {
        self.ctx = app_ctx;
    }

    #[inline(always)]
    fn salsa_ctx(&self) -> &SalsaAppContext<AppEvent, Error> {
        &self.ctx
    }
}

impl Global {
    pub fn new(cfg: Config) -> Self {
        Self {
            ctx: Default::default(),
            cfg,
        }
    }
}

#[derive(Default)]
pub struct State<'a> {
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

pub fn init(state: &mut State, ctx: &mut Global) -> Result<(), Error> {
    Ok(())
}

fn select_next(state: &mut State) {
    state.session_list_state.select_next();
}

fn select_previous(state: &mut State) {
    state.session_list_state.select_previous();
}

fn select_first(state: &mut State) {
    state.session_list_state.select_first();
}

fn select_last(state: &mut State) {
    state.session_list_state.select_last();
}

fn exit(state: &mut State) {
    state.exit = true;
}

pub fn event(event: &Event, state: &mut State, ctx: &mut Global) -> Result<Control<Event>, Error> {
    match event {
        Event::Key(key_event)
            if key_event.modifiers == KeyModifiers::CONTROL
                && key_event.code == KeyCode::Char('c') =>
        {
            exit(state);
        }
        Event::Key(key_event) => handle_key_event(state, *key_event),
        _ => {}
    };
    Ok(())
}

fn handle_key_event(state: &mut State, key_event: KeyEvent) {
    match state.mode {
        Mode::Main => {
            state.sessions = tmux::list_sessions().unwrap();
            match key_event.code {
                KeyCode::Char('q') => exit(state),
                KeyCode::Down | KeyCode::Char('j') => select_next(state),
                KeyCode::Up | KeyCode::Char('k') => select_previous(state),
                KeyCode::Char('g') => select_first(state),
                KeyCode::Char('G') => select_last(state),
                KeyCode::Char('a') => state.mode = Mode::Create,
                KeyCode::Char('r') => state.mode = Mode::Rename,
                KeyCode::Char('d') => state.mode = Mode::Delete,
                KeyCode::Enter => {
                    if let Some(index) = state.session_list_state.selected() {
                        tmux::switch_session(&state.sessions[index].name).unwrap()
                    };
                }
                _ => {}
            }
        }
        Mode::Create => match key_event.code {
            KeyCode::Esc => {
                state.text_area = TextArea::default();
                state.mode = Mode::Main;
            }
            KeyCode::Enter => {
                tmux::create_session(&state.text_area.lines().join("\n")).unwrap();
                state.text_area = TextArea::default();
                state.mode = Mode::Main
            }
            _ => _ = state.text_area.input(key_event),
        },
        Mode::Rename => match key_event.code {
            KeyCode::Esc => {
                state.text_area = TextArea::default();
                state.mode = Mode::Main;
            }
            KeyCode::Enter => {
                if let Some(index) = state.session_list_state.selected() {
                    tmux::rename_session(
                        &state.sessions[index].name,
                        &state.text_area.lines().join(""),
                    )
                    .unwrap()
                };
                state.text_area = TextArea::default();
                state.mode = Mode::Main;
            }
            _ => _ = state.text_area.input(key_event),
        },
        Mode::Delete => match key_event.code {
            KeyCode::Char('y') | KeyCode::Enter => {
                if let Some(index) = state.session_list_state.selected() {
                    tmux::delete_session(&state.sessions[index].name).unwrap()
                };
                state.mode = Mode::Main;
            }
            KeyCode::Char('n') | KeyCode::Esc => state.mode = Mode::Main,
            _ => {}
        },
    };
}
