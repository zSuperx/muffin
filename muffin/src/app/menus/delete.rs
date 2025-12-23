use super::traits::Menu;
use crate::app::{
    app::{AppState, EventHandler},
    menus::utils::{centered_fixed_rect, make_instructions},
};
use ratatui::{
    prelude::{self, Buffer, Constraint, Layout},
    style::{Style, Stylize},
    symbols::border,
    text::Line,
    widgets::{
        Block, Borders, Clear, HighlightSpacing, List, ListItem, ListState, Paragraph,
        StatefulWidget, Widget, Wrap,
    },
};
use tui_textarea::TextArea;

pub struct DeleteMenu<'a> {
    text_area: TextArea<'a>,
    handler: &'a EventHandler,
    notification: Option<String>,
}

impl<'a> Menu for DeleteMenu<'a> {
    fn render(&mut self, area: prelude::Rect, buf: &mut Buffer, state: &AppState) {
        let area = centered_fixed_rect(area, 40, 15);
        Clear.render(area, buf);

        let block = Block::bordered().border_style(Style::new().red());
        let inner_area = block.inner(area);

        let [title_area, instructions_area] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(1)])
                .vertical_margin(1)
                .horizontal_margin(1)
                .areas(inner_area);

        let index = state.session_list_state.selected().unwrap();

        // Render title
        {
            let content = match self.notification.clone() {
                Some(msg) => msg,
                _ => format!("Delete session '{}'?", state.sessions[index].name),
            };

            Line::from(content.red()).centered().render(title_area, buf);
        }

        // Render instructions
        {
            let instructions = vec![("y/enter", "delete"), ("n/esc", "cancel")];

            Paragraph::new(make_instructions(instructions))
                .wrap(Wrap { trim: true })
                .centered()
                .render(instructions_area, buf);
        }

        block.render(area, buf);
    }

    fn handle_event(&mut self, event: crate::app::app::AppEvent) {
        todo!()
    }
}
