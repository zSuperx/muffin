use super::Menu;
use crate::app::{
    driver::{AppEvent, AppState, AppMode},
    utils::{centered_fixed_rect, make_instructions, send_timed_notification},
};
use crossterm::event::KeyCode;
use ratatui::{
    prelude::{self, Buffer, Constraint, Layout},
    style::{Style, Stylize},
    text::Line,
    widgets::{Block, Clear, Paragraph, StatefulWidget, Widget, Wrap},
};
use tui_textarea::TextArea;

#[derive(Default)]
pub struct RenameMenu<'a> {
    text_area: TextArea<'a>,
    notification: Option<String>,
}

impl<'a> StatefulWidget for &mut RenameMenu<'a> {
    type State = AppState;

    fn render(self, area: prelude::Rect, buf: &mut Buffer, state: &mut AppState) {
        let area = centered_fixed_rect(area, 40, 15);
        Clear.render(area, buf);

        let block = Block::bordered().border_style(Style::new().light_green());
        let inner_area = block.inner(area);

        let [title_area, input_area, instructions_area] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .vertical_margin(1)
        .horizontal_margin(1)
        .areas(inner_area);

        // Render title
        {
            let index = state.selected_session.unwrap();
            let content = match self.notification.clone() {
                Some(msg) => msg,
                _ => format!("Rename session '{}' to...", state.sessions[index].name),
            };

            Line::from(content.light_green())
                .centered()
                .render(title_area, buf);
        }

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
}

impl<'a> Menu for RenameMenu<'a> {
    fn handle_event(&mut self, event: AppEvent, state: &mut AppState) {
        match event {
            AppEvent::Key(key_event) => match key_event.code {
                KeyCode::Esc => {
                    self.text_area = TextArea::default();
                    state.mode = AppMode::Sessions;
                }
                KeyCode::Enter => {
                    if let Some(index) = state.selected_session {
                        match tmux::rename_session(
                            &state.sessions[index].name,
                            &self.text_area.lines().join(""),
                        ) {
                            Ok(_) => {
                                self.text_area = TextArea::default();
                                state.mode = AppMode::Sessions;
                            }
                            Err(s) => send_timed_notification(&state.event_handler, s),
                        }
                    };
                }
                _ => _ = self.text_area.input(key_event),
            },
            AppEvent::ShowNotification(msg) => self.notification = Some(msg),
            AppEvent::ClearNotification => self.notification = None,
            _ => {}
        }
    }
}
