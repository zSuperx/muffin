use ratatui::{DefaultTerminal, buffer::Buffer, layout::Rect, widgets::StatefulWidget};

use crate::app::app::AppState;

use super::super::app::AppEvent;

pub trait Menu {
    fn handle_event(&mut self, event: AppEvent, state: &mut AppState, terminal: &mut DefaultTerminal);
}
