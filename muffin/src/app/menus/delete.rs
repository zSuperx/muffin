use super::traits::Menu;
use crate::app::{
    app::{AppEvent, AppState, Mode},
    menus::utils::{centered_fixed_rect, make_instructions, send_timed_notification},
};
use crossterm::event::KeyCode;
use ratatui::{
    DefaultTerminal, prelude::{Buffer, Constraint, Layout, Rect}, style::{Style, Stylize}, text::Line, widgets::{Block, Clear, Paragraph, StatefulWidget, Widget, Wrap}
};
use tui_textarea::TextArea;

#[derive(Default)]
pub struct DeleteMenu<'a> {
    text_area: TextArea<'a>,
    notification: Option<String>,
}

impl<'a> StatefulWidget for &mut DeleteMenu<'a> {
    type State = AppState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut AppState) {
        let area = centered_fixed_rect(area, 40, 15);
        Clear.render(area, buf);

        let block = Block::bordered().border_style(Style::new().red());
        let inner_area = block.inner(area);

        let [title_area, instructions_area] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(1)])
                .vertical_margin(1)
                .horizontal_margin(1)
                .areas(inner_area);

        // Render title
        {
            let index = state.selected_session.unwrap();
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
}

impl<'a> Menu for DeleteMenu<'a> {
    fn handle_event(&mut self, event: AppEvent, state: &mut AppState, terminal: &mut DefaultTerminal) {
        match event {
            AppEvent::Tick => _ = terminal.draw(|frame| frame.render_stateful_widget(self, frame.area(), state)).unwrap(),
            AppEvent::Key(key_event) => match key_event.code {
                KeyCode::Char('y') | KeyCode::Enter => {
                    if let Some(index) = state.selected_session {
                        match tmux_helper::delete_session(&state.sessions[index].name) {
                            Ok(_) => {
                                self.text_area = TextArea::default();
                                state.mode = Mode::Main;
                            }
                            Err(s) => send_timed_notification(&state.event_handler, s),
                        }
                    };
                }
                KeyCode::Char('n') | KeyCode::Esc => state.mode = Mode::Main,
                _ => {}
            },
            AppEvent::ShowNotification(msg) => self.notification = Some(msg),
            AppEvent::ClearNotification => self.notification = None,
            _ => {}
        }
    }
}
