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

pub struct CreateMenu<'a> {
    handler: &'a EventHandler,
    text_area: TextArea<'a>,
    notification: Option<String>,
}

impl<'a> Menu for CreateMenu<'a> {
    fn render(&mut self, area: prelude::Rect, buf: &mut Buffer, state: &AppState) {
        let area = centered_fixed_rect(area, 40, 15);
        Clear.render(area, buf);

        let block = Block::bordered().border_style(Style::new().blue());
        let inner_area = block.inner(area);

        let [title_area, input_area, instructions_area] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .vertical_margin(1)
        .horizontal_margin(1)
        .areas(inner_area);

        {
            let content = match self.notification.clone() {
                Some(msg) => msg,
                _ => "Name new session".to_string(),
            };

            Line::from(content.blue())
                .centered()
                .render(title_area, buf);
        }

        // Render input field
        {
            let [first_char, rest] =
                Layout::horizontal([Constraint::Length(2), Constraint::Fill(1)])
                    .horizontal_margin(3)
                    .areas(input_area);

            "> ".blue().render(first_char, buf);

            self.text_area.set_placeholder_text("start typing!");
            self.text_area
                .set_placeholder_style(Style::new().dark_gray());
            self.text_area.render(rest, buf);
        }

        // Render instructions
        {
            let instructions = vec![("esc", "cancel"), ("enter", "create")];

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
