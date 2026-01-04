use super::Menu;
use crate::app::{
    driver::{AppEvent, AppState, AppMode},
    utils::{centered_fixed_rect, make_instructions, send_timed_notification},
};
use crossterm::event::KeyCode;
use ratatui::{
    prelude::{Buffer, Constraint, Layout, Rect},
    style::{Style, Stylize},
    text::Line,
    widgets::{Block, Clear, Paragraph, StatefulWidget, Widget, Wrap},
};
use tui_textarea::TextArea;

#[derive(Default)]
pub struct CreateMenu<'a> {
    text_area: TextArea<'a>,
    notification: Option<String>,
}

impl<'a> StatefulWidget for &mut CreateMenu<'a> {
    type State = AppState;

    fn render(self, area: Rect, buf: &mut Buffer, _state: &mut AppState) {
        let area = centered_fixed_rect(area, 40, 15);

        let block = Block::bordered().border_style(Style::new().blue());
        let inner_area = block.inner(area);
        Clear.render(area, buf);

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
}

impl<'a> Menu for CreateMenu<'a> {
    fn handle_event(&mut self, event: AppEvent, state: &mut AppState) {
        match event {
            AppEvent::Key(key_event) => match key_event.code {
                KeyCode::Esc => {
                    self.text_area = TextArea::default();
                    state.mode = AppMode::Sessions;
                }
                KeyCode::Enter => {
                    match tmux::create_session(&self.text_area.lines().join("\n")) {
                        Ok(_) => {
                            self.text_area = TextArea::default();
                            state.mode = AppMode::Sessions;
                        }
                        Err(s) => send_timed_notification(&state.event_handler, s),
                    }
                }
                _ => _ = self.text_area.input(key_event),
            },
            AppEvent::ShowNotification(msg) => self.notification = Some(msg),
            AppEvent::ClearNotification => self.notification = None,
            _ => {}
        }
    }
}
