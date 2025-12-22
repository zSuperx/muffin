use std::vec;

use crate::app::{App, Mode};
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
                &mut self.session_list_state,
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

    pub fn render_create(&mut self, area: Rect, buf: &mut Buffer) {
        let area = centered_rect(area, 70, 50);
        Clear.render(area, buf);

        let block = Block::bordered().border_style(Style::new().blue());
        let inner_area = block.inner(area);

        let [title_area, input_area, instructions_area] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Length(2),
            Constraint::Length(1),
        ])
        .vertical_margin(1)
        .horizontal_margin(1)
        .areas(inner_area);

        Line::from("Name new session".blue())
            .centered()
            .render(title_area, buf);

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

    pub fn render_rename(&mut self, area: Rect, buf: &mut Buffer) {
        let area = centered_rect(area, 70, 50);
        Clear.render(area, buf);

        let block = Block::bordered().border_style(Style::new().light_green());
        let inner_area = block.inner(area);

        let [title_area, input_area, instructions_area] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Length(2),
            Constraint::Length(1),
        ])
        .vertical_margin(1)
        .horizontal_margin(1)
        .areas(inner_area);

        let index = self.session_list_state.selected().unwrap();

        Line::from(format!("Rename session '{}' to...", self.sessions[index].name).light_green())
            .centered()
            .render(title_area, buf);

        // Render input field
        {
            let [first_char, rest] =
                Layout::horizontal([Constraint::Length(2), Constraint::Fill(1)])
                    .horizontal_margin(3)
                    .areas(input_area);

            "> ".light_green().render(first_char, buf);

            self.text_area.set_placeholder_text("start typing!");
            self.text_area
                .set_placeholder_style(Style::new().dark_gray());
            self.text_area.render(rest, buf);
        }

        // Render instructions
        {
            let instructions = vec![("esc", "cancel"), ("enter", "rename")];

            Paragraph::new(make_instructions(instructions))
                .wrap(Wrap { trim: true })
                .centered()
                .render(instructions_area, buf);
        }

        block.render(area, buf);
    }

    pub fn render_delete(&mut self, area: Rect, buf: &mut Buffer) {
        let area = centered_rect(area, 70, 50);
        Clear.render(area, buf);

        let block = Block::bordered().border_style(Style::new().red());
        let inner_area = block.inner(area);

        let [title_area, instructions_area] = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .vertical_margin(1)
        .horizontal_margin(1)
        .areas(inner_area);

        let index = self.session_list_state.selected().unwrap();

        // Render title
        {
            let content = match self.notification.clone() {
                Some(msg) => msg,
                None => format!("Delete session '{}'?", self.sessions[index].name),
            };

            Line::from(content.red())
                .centered()
                .render(title_area, buf);
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

/// helper function to create a centered rect using up certain percentage of the available rect `r`
pub fn centered_rect(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}

pub fn centered_fixed_rect(r: Rect, width: u16, height: u16) -> Rect {
    let [_, popup_area, _] = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(height),
        Constraint::Fill(1),
    ])
    .areas(r);

    Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Length(width),
        Constraint::Fill(1),
    ])
    .split(popup_area)[1]
}

fn make_instructions<'a>(instructions: Vec<(&'a str, &'a str)>) -> Line<'a> {
    Line::from(
        instructions
            .iter()
            .flat_map(|(key, desc)| {
                vec![format!(" {}", key).gray(), format!(":{desc} ").dark_gray()]
            })
            .collect::<Vec<Span>>(),
    )
}
