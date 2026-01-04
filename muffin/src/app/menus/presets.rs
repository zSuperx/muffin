use super::Menu;
use crate::app::{
    driver::{AppEvent, AppState, AppMode},
    utils::{make_instructions, send_timed_notification},
};
use crossterm::event::KeyCode;
use ratatui::{
    prelude::{Buffer, Constraint, Layout, Rect},
    style::{Style, Stylize},
    symbols::border,
    text::Line,
    widgets::{
        Block, Borders, Clear, HighlightSpacing, List, ListItem, ListState, Paragraph,
        StatefulWidget, Widget, Wrap,
    },
};

pub struct PresetsMenu {
    list_state: ListState,
    notification: Option<String>,
}

impl PresetsMenu {
    pub fn new(index: Option<usize>) -> Self {
        let mut list_state = ListState::default();
        list_state.select(index);
        Self {
            list_state,
            notification: None,
        }
    }

    pub fn select_next(&mut self, length: usize) -> Option<usize> {
        self.list_state.select_next();
        self.list_state
            .selected()
            .map(|idx| idx.clamp(0, length.saturating_sub(1)))
    }

    pub fn select_previous(&mut self, length: usize) -> Option<usize> {
        self.list_state.select_previous();
        self.list_state
            .selected()
            .map(|idx| idx.clamp(0, length.saturating_sub(1)))
    }

    pub fn select_first(&mut self, length: usize) -> Option<usize> {
        self.list_state.select_first();
        self.list_state
            .selected()
            .map(|idx| idx.clamp(0, length.saturating_sub(1)))
    }

    pub fn select_middle(&mut self, length: usize) -> Option<usize> {
        if length > 0 {
            let new_index = (length.saturating_sub(1)).div_ceil(2);
            self.list_state.select(Some(new_index));
        }
        self.list_state
            .selected()
            .map(|idx| idx.clamp(0, length.saturating_sub(1)))
    }

    pub fn select_last(&mut self, length: usize) -> Option<usize> {
        self.list_state.select_last();
        self.list_state
            .selected()
            .map(|idx| idx.clamp(0, length.saturating_sub(1)))
    }
}

impl StatefulWidget for &mut PresetsMenu {
    type State = AppState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut AppState) {
        Clear.render(area, buf);
        let block = Block::bordered().border_set(border::THICK);

        let inner_area = block.inner(area);

        let [
            title_area,
            notification_area,
            presets_area,
            instructions_area,
        ] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Max(2),
            Constraint::Fill(1),
            Constraint::Length(2),
        ])
        .spacing(1)
        .areas(inner_area);

        // Render title
        {
            Paragraph::new(Line::from("Presets").underlined().bold().italic())
                .centered()
                .block(Block::new().borders(Borders::BOTTOM))
                .render(title_area, buf);
        }

        // Render notification
        {
            let content = match self.notification.clone() {
                Some(msg) => msg.red(),
                None => format!("Reading presets from {}", state.presets_path).into(),
            };
            Paragraph::new(Line::from(content.italic()))
                .centered()
                .wrap(Wrap { trim: false })
                .render(notification_area, buf);
        }

        // Render presets
        {
            let sessions_width = 20;
            let [_, presets_area, running_status_area, _] = Layout::horizontal([
                Constraint::Fill(1),
                Constraint::Length(sessions_width),
                Constraint::Length(11),
                Constraint::Fill(1),
            ])
            .areas(presets_area);

            let presets = state
                .presets
                .values()
                .map(|s| {
                    let truncated_name = if s.name.len() > sessions_width as usize - 8 {
                        let mut name = s.name.clone();
                        name.truncate(sessions_width as usize - 11);
                        format!("{}...", name)
                    } else {
                        s.name.clone()
                    };
                    let text = format!("{:>2}  - {}", s.windows.len(), truncated_name);
                    let item = Line::from(text.clone());
                    ListItem::new(item)
                })
                .collect::<Vec<ListItem>>();

            StatefulWidget::render(
                List::new(presets)
                    .highlight_symbol("")
                    .highlight_spacing(HighlightSpacing::Always)
                    .highlight_style(Style::new().italic().bold().cyan()),
                presets_area,
                buf,
                &mut self.list_state,
            );

            Paragraph::new(
                state
                    .presets
                    .values()
                    .map(|s| if s.running { "   running" } else { "" })
                    .collect::<Vec<&str>>()
                    .join("\n"),
            )
            .green()
            .render(running_status_area, buf);
        }

        // Render instructions
        {
            let instructions = vec![
                ("enter", "launch"),
                ("q", "quit"),
                ("j/↓", "next"),
                ("k/↑", "prev"),
                ("tab", "view sessions"),
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

impl Menu for PresetsMenu {
    fn handle_event(&mut self, event: AppEvent, state: &mut AppState) {
        match event {
            AppEvent::Key(key_event) => match key_event.code {
                // Movement
                KeyCode::Down | KeyCode::Char('j') => {
                    state.selected_preset = self.select_next(state.presets.len())
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    state.selected_preset = self.select_previous(state.presets.len())
                }
                KeyCode::Char('g') => {
                    state.selected_preset = self.select_first(state.presets.len())
                }
                KeyCode::Char('M') => {
                    state.selected_preset = self.select_middle(state.presets.len())
                }
                KeyCode::Char('G') => state.selected_preset = self.select_last(state.presets.len()),

                // Mode switching
                KeyCode::Tab => state.mode = AppMode::Sessions,

                // Control
                KeyCode::Char('q') => state.exit = true,
                KeyCode::Enter => {
                    if let Some(index) = state.selected_preset {
                        match tmux::spawn_preset(state.presets.values().nth(index).unwrap()) {
                            Ok(_) => {
                                state.mode = AppMode::Sessions;
                            }
                            Err(s) => send_timed_notification(&state.event_handler, s),
                        }
                    };
                }
                _ => {}
            },
            AppEvent::ShowNotification(msg) => self.notification = Some(msg),
            AppEvent::ClearNotification => self.notification = None,
            _ => {}
        }
    }
}
