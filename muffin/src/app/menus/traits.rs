use ratatui::{buffer::Buffer, layout::Rect};

use crate::app::app::AppState;

use super::super::app::AppEvent;

pub trait Menu {
    fn handle_event(&mut self, event: AppEvent);
    fn render(&mut self, area: Rect, buf: &mut Buffer, state: &AppState);
}
