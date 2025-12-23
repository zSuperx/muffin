use super::traits::Menu;
use crate::app::{
    app::{AppEvent, AppState, Mode},
    menus::utils::make_instructions,
};
use crossterm::event::KeyCode;
use ratatui::{
    DefaultTerminal, prelude::{Buffer, Constraint, Layout, Rect}, style::{Style, Stylize}, symbols::border, text::Line, widgets::{
        Block, Borders, HighlightSpacing, List, ListItem, ListState, Paragraph, StatefulWidget,
        Widget, Wrap,
    }
};

#[derive(Default)]
pub struct SessionsMenu {
    list_state: ListState,
    notification: Option<String>,
}

impl SessionsMenu {
    pub fn select_next(&mut self) -> Option<usize> {
        self.list_state.select_next();
        self.list_state.selected()
    }

    pub fn select_previous(&mut self) -> Option<usize> {
        self.list_state.select_previous();
        self.list_state.selected()
    }

    pub fn select_first(&mut self) -> Option<usize> {
        self.list_state.select_first();
        self.list_state.selected()
    }

    pub fn select_middle(&mut self, length: usize) -> Option<usize> {
        if length > 0 {
            let new_index = (length - 1).div_ceil(2);
            self.list_state.select(Some(new_index));
        }
        self.list_state.selected()
    }

    pub fn select_last(&mut self) -> Option<usize> {
        self.list_state.select_last();
        self.list_state.selected()
    }
}

impl StatefulWidget for &mut SessionsMenu {
    type State = AppState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut AppState) {
        let block = Block::bordered().border_set(border::THICK);

        let inner_area = block.inner(area);

        let [
            title_area,
            notification_area,
            sessions_area,
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
            Paragraph::new(Line::from("Sessions").underlined().bold().italic())
                .centered()
                .block(Block::new().borders(Borders::BOTTOM))
                .render(title_area, buf);
        }

        // Render sessions
        {
            let max_name_len = state
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

            let sessions = state
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

impl Menu for SessionsMenu {
    fn handle_event(&mut self, event: AppEvent, state: &mut AppState, terminal: &mut DefaultTerminal) {
        match event {
            AppEvent::Error => todo!(),
            AppEvent::Tick => _ = terminal.draw(|frame| frame.render_stateful_widget(self, frame.area(), state)).unwrap(),
            AppEvent::Key(key_event) => match key_event.code {
                // Movement
                KeyCode::Down | KeyCode::Char('j') => state.selected_session = self.select_next(),
                KeyCode::Up | KeyCode::Char('k') => state.selected_session = self.select_previous(),
                KeyCode::Char('g') => state.selected_session = self.select_first(),
                KeyCode::Char('M') => {
                    state.selected_session = self.select_middle(state.sessions.len())
                }
                KeyCode::Char('G') => state.selected_session = self.select_last(),

                // Mode switching
                KeyCode::Char('a') => state.mode = Mode::Create,
                KeyCode::Char('r') => state.mode = Mode::Rename,
                KeyCode::Char('d') => state.mode = Mode::Delete,

                // Control
                KeyCode::Char('q') => state.exit = true,
                KeyCode::Enter => {
                    if let Some(index) = state.selected_session {
                        tmux_helper::switch_session(&state.sessions[index].name).unwrap()
                    };
                }
                _ => {}
            },
            AppEvent::ShowNotification(msg) => self.notification = Some(msg),
            AppEvent::ClearNotification => self.notification = None,
        }
    }
}
