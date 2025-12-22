use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tui_textarea::TextArea;

use super::super::app::*;
use crate::tmux::tmux;

impl<'a> App<'a> {
    pub fn handle_key(&mut self, key_event: KeyEvent) {
        if key_event.modifiers == KeyModifiers::CONTROL && key_event.code == KeyCode::Char('c') {
            self.exit();
            return;
        }
        match self.mode {
            Mode::Main => self.handle_key_main(key_event),
            Mode::Create => self.handle_key_create(key_event),
            Mode::Rename => self.handle_key_rename(key_event),
            Mode::Delete => self.handle_key_delete(key_event),
        };
    }

    fn handle_key_main(&mut self, key_event: KeyEvent) {
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

    fn handle_key_create(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Esc => {
                self.text_area = TextArea::default();
                self.mode = Mode::Main;
            }
            KeyCode::Enter => {
                match tmux::create_session(&self.text_area.lines().join("\n")) {
                    Ok(_) => {
                        self.text_area = TextArea::default();
                        self.mode = Mode::Main;
                    }
                    Err(s) => self.send_timed_notification(Mode::Create, s),
                }
            }
            _ => _ = self.text_area.input(key_event),
        }
    }

    fn handle_key_rename(&mut self, key_event: KeyEvent) {
        match key_event.code {
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
                        Err(s) => self.send_timed_notification(Mode::Rename, s),
                    }
                };
            }
            _ => _ = self.text_area.input(key_event),
        }
    }

    fn handle_key_delete(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('y') | KeyCode::Enter => {
                if let Some(index) = self.session_list_state.selected() {
                    match tmux::delete_session(&self.sessions[index].name) {
                        Ok(_) => {
                            self.text_area = TextArea::default();
                            self.mode = Mode::Main;
                        }
                        Err(s) => self.send_timed_notification(Mode::Delete, s),
                    }
                };
            }
            KeyCode::Char('n') | KeyCode::Esc => self.mode = Mode::Main,
            _ => {}
        }
    }
}
