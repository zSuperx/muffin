use super::traits::Menu;
use crate::app::{app::EventHandler, menus::utils::make_instructions};
use ratatui::{
    prelude::{self, Buffer, Constraint, Layout},
    style::{Style, Stylize},
    symbols::border,
    text::Line,
    widgets::{
        Block, Borders, HighlightSpacing, List, ListItem, ListState, Paragraph, StatefulWidget,
        Widget, Wrap,
    },
};
use tmux_helper::Session;

pub struct SessionsMenu<'a> {
    sessions: Vec<Session>,
    list_state: ListState,
    handler: &'a EventHandler,
}

impl<'a> Widget for &mut SessionsMenu<'a> {
    fn render(self, area: prelude::Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let block = Block::bordered().border_set(border::THICK);

        let inner_area = block.inner(area);

        let [title_area, sessions_area, instructions_area] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(2),
        ])
        .spacing(1)
        .areas(inner_area);

        // Render title
        {
            Paragraph::new(Line::from("Sessions").underlined().bold().italic())
                .centered()
                .block(Block::new().borders(Borders::BOTTOM))
                .render(title_area, buf);
        }

        // Render sessions
        {
            let max_name_len = self
                .sessions
                .iter()
                .map(|s| s.name.len())
                .max()
                .unwrap_or(30)
                .max(10)
                + 2
                + 2
                + 5
                + 9;

            let [_, sessions_area, _] = Layout::horizontal([
                Constraint::Fill(1),
                Constraint::Length(max_name_len.try_into().unwrap_or(30)),
                Constraint::Fill(1),
            ])
            .areas(sessions_area);

            let sessions = self
                .sessions
                .iter()
                .map(|s| {
                    let text = format!(
                        "{:>2} 󱂬 - {:<10} {:<9}",
                        s.windows,
                        s.name,
                        if s.active { " 󰞓 active" } else { "" }
                    );
                    let mut item = Line::from(text.clone());
                    if s.active {
                        item = item.green();
                    }
                    ListItem::new(item)
                })
                .collect::<Vec<ListItem>>();

            StatefulWidget::render(
                List::new(sessions)
                    .highlight_symbol("")
                    .highlight_spacing(HighlightSpacing::Always)
                    .highlight_style(Style::new().italic().bold().cyan()),
                sessions_area,
                buf,
                &mut self.list_state,
            );
        }

        // Render instructions
        {
            let instructions = vec![
                ("a", "create"),
                ("r", "rename"),
                ("enter", "switch"),
                ("q", "quit"),
                ("j/↓", "next"),
                ("k/↑", "prev"),
                ("g", "first"),
                ("G", "last"),
            ];

            Paragraph::new(make_instructions(instructions))
                .wrap(Wrap { trim: true })
                .dark_gray()
                .centered()
                .render(instructions_area, buf);
        }

        block.render(area, buf);
    }
}

impl<'a> Menu for &mut SessionsMenu<'a> {
    fn handle_event(&mut self, event: super::super::app::AppEvent) {}
}
