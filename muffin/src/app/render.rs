use std::vec;

use super::app::{App, Mode};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Flex, Layout, Rect},
    style::{Style, Stylize},
    symbols::border,
    text::{Line, Span},
    widgets::{
        Block, Borders, Clear, HighlightSpacing, List, ListItem, Paragraph, StatefulWidget, Widget,
        Wrap,
    },
};

impl<'a> App<'a> {
    pub fn render_sessions(&mut self, area: Rect, buf: &mut Buffer) {
    }

    pub fn render_create(&mut self, area: Rect, buf: &mut Buffer) {
    }

    pub fn render_rename(&mut self, area: Rect, buf: &mut Buffer) {
    }

    pub fn render_delete(&mut self, area: Rect, buf: &mut Buffer) {
    }
}

impl<'a> Widget for &mut App<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Always render the sessions UI
        self.render_sessions(area, buf);

        match self.mode {
            Mode::Main => {}
            Mode::Create => self.render_create(area, buf),
            Mode::Rename => self.render_rename(area, buf),
            Mode::Delete => self.render_delete(area, buf),
        }
    }
}

