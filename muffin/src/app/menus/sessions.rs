use super::Menu;
use crate::app::{
    driver::{AppEvent, AppMode, AppState},
    utils::{make_instructions, send_timed_notification},
};
use crossterm::event::KeyCode;
use ratatui::{
    prelude::{Buffer, Constraint, Layout, Rect},
    style::{Style, Stylize},
    symbols::border,
    text::{Line, Span, Text},
    widgets::{
        Block, Borders, Clear, HighlightSpacing, List, ListItem, ListState, Paragraph,
        StatefulWidget, Widget, Wrap,
    },
};
use tmux::Session;
use tui_textarea::TextArea;

pub struct SessionsMenu<'a> {
    list_state: ListState,
    notification: Option<String>,
    displayed_sessions: Vec<usize>,
    search_bar: TextArea<'a>,
    mode: MenuMode,
}

enum MenuMode {
    SearchInsert,
    Normal,
}

impl<'a> SessionsMenu<'a> {
    pub fn new(total_session: usize, index: Option<usize>) -> Self {
        let mut list_state = ListState::default();
        list_state.select(index);
        Self {
            list_state,
            notification: None,
            displayed_sessions: (0..total_session).collect(),
            search_bar: TextArea::default(),
            mode: MenuMode::Normal,
        }
    }

    pub fn select_next(&mut self, state: &mut AppState) -> Option<usize> {
        self.list_state.select_next();
        self.verify_index(
            self.list_state
                .selected()
                .map(|idx| idx.clamp(0, self.displayed_sessions.len().saturating_sub(1))),
            state,
        )
    }

    pub fn select_previous(&mut self, state: &mut AppState) -> Option<usize> {
        self.list_state.select_previous();
        self.verify_index(
            self.list_state
                .selected()
                .map(|idx| idx.clamp(0, self.displayed_sessions.len().saturating_sub(1))),
            state,
        )
    }

    pub fn select_first(&mut self, state: &mut AppState) -> Option<usize> {
        self.list_state.select_first();
        self.verify_index(
            self.list_state
                .selected()
                .map(|idx| idx.clamp(0, self.displayed_sessions.len().saturating_sub(1))),
            state,
        )
    }

    pub fn select_middle(&mut self, state: &mut AppState) -> Option<usize> {
        if self.displayed_sessions.len() > 0 {
            let new_index = (self.displayed_sessions.len().saturating_sub(1)).div_ceil(2);
            self.list_state.select(Some(new_index));
        }
        self.verify_index(
            self.list_state
                .selected()
                .map(|idx| idx.clamp(0, self.displayed_sessions.len().saturating_sub(1))),
            state,
        )
    }

    pub fn select_last(&mut self, state: &mut AppState) -> Option<usize> {
        self.list_state.select_last();
        self.verify_index(
            self.list_state
                .selected()
                .map(|idx| idx.clamp(0, self.displayed_sessions.len().saturating_sub(1))),
            state,
        )
    }

    fn verify_index(&mut self, x: Option<usize>, state: &mut AppState) -> Option<usize> {
        x.and_then(|idx| {
            if self
                .displayed_sessions
                .get(idx)
                .is_some_and(|&i| i < state.sessions.len())
            {
                Some(idx)
            } else {
                None
            }
        })
    }
}

impl<'a> StatefulWidget for &mut SessionsMenu<'a> {
    type State = AppState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut AppState) {
        Clear.render(area, buf);
        let block = Block::bordered().border_set(border::THICK);

        let inner_area = block.inner(area);

        let [
            title_area,
            subtitle_area,
            search_area,
            sessions_area,
            instructions_area,
        ] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Length(1),
            Constraint::Length(1),
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

        // Render top content
        {
            match self.mode {
                // In search insert mode, render message
                // then filter
                MenuMode::SearchInsert => {
                    Paragraph::new(Line::from("Type to filter...".italic()))
                        .centered()
                        .render(subtitle_area, buf);

                    let [_, text_area, _] = Layout::horizontal([
                        Constraint::Fill(1),
                        Constraint::Length(30),
                        Constraint::Fill(1),
                    ])
                    .areas(search_area);
                    let [first, rest] =
                        Layout::horizontal([Constraint::Length(8), Constraint::Fill(1)])
                            .horizontal_margin(3)
                            .areas(text_area);

                    "Filter: ".render(first, buf);
                    self.search_bar
                        .set_cursor_style(Style::default().on_white());
                    self.search_bar.render(rest, buf);
                }
                // In normal mode, render notification (if any, else message)
                // then filter (if any)
                MenuMode::Normal => {
                    let content = match self.notification.clone() {
                        Some(msg) => msg.red(),
                        None => "Select a session!".into(),
                    };
                    Paragraph::new(Line::from(content.italic()))
                        .centered()
                        .render(subtitle_area, buf);

                    if !self.search_bar.is_empty() {
                        let [_, text_area, _] = Layout::horizontal([
                            Constraint::Fill(1),
                            Constraint::Length(30),
                            Constraint::Fill(1),
                        ])
                        .areas(search_area);
                        let [first, rest] =
                            Layout::horizontal([Constraint::Length(8), Constraint::Fill(1)])
                                .horizontal_margin(3)
                                .areas(text_area);

                        "Filter: ".render(first, buf);
                        self.search_bar.set_cursor_style(Style::default());
                        self.search_bar.render(rest, buf);
                    }
                }
            }
        }

        // Render sessions
        {
            let sessions_width = 20;
            let [_, sessions_area, active_status_area, _] = Layout::horizontal([
                Constraint::Fill(1),
                Constraint::Length(sessions_width),
                Constraint::Max(10),
                Constraint::Fill(1),
            ])
            .areas(sessions_area);

            let sessions = self
                .displayed_sessions
                .iter()
                .filter_map(|idx| {
                    let Some(session) = &state.sessions.get(*idx) else {
                        return None;
                    };
                    let truncated_name = if session.name.len() > sessions_width as usize - 8 {
                        let mut name = session.name.clone();
                        name.truncate(sessions_width as usize - 11);
                        format!("{}...", name)
                    } else {
                        session.name.clone()
                    };
                    let text = format!("{:>2}  - {}", session.windows, truncated_name);
                    let mut item = Line::from(text.clone());
                    if session.active {
                        item = item.green();
                    }
                    Some(ListItem::new(item))
                })
                .collect::<Vec<ListItem>>();

            Paragraph::new(Text::from(
                self.displayed_sessions
                    .iter()
                    .filter_map(|idx| {
                        let Some(session) = &state.sessions.get(*idx) else {
                            return None;
                        };
                        Some(Line::from(if session.active {
                            // Color ACTIVE (attached & current terminal) green
                            "   active".green()
                        } else if session.attached {
                            // Color ATTACHED (attached in diff terminal) dark gray
                            "  attached".dark_gray()
                        } else {
                            "\n".into()
                        }))
                    })
                    .collect::<Vec<Line>>(),
            ))
            .green()
            .render(active_status_area, buf);

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
                ("enter", "switch"),
                ("q", "quit"),
                ("j/↓", "next"),
                ("k/↑", "prev"),
                ("a", "create"),
                ("r", "rename"),
                ("/", "search"),
                ("tab", "view presets"),
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

impl<'a> Menu for SessionsMenu<'a> {
    fn pre_render(&mut self, state: &mut AppState) {
        self.displayed_sessions = if self.search_bar.is_empty() {
            (0..state.sessions.len()).collect()
        } else {
            let search_query = self.search_bar.lines().join("");
            state
                .sessions
                .iter()
                .enumerate()
                .filter_map(|(idx, Session { name, .. })| {
                    name.to_ascii_lowercase()
                        .contains(&search_query.to_ascii_lowercase())
                        .then_some(idx)
                })
                .collect()
        }
    }

    fn handle_event(&mut self, event: AppEvent, state: &mut AppState) {
        match event {
            AppEvent::Key(key_event) => match self.mode {
                MenuMode::Normal => match key_event.code {
                    // Movement
                    KeyCode::Down | KeyCode::Char('j') => {
                        state.selected_session = self.select_next(state)
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        state.selected_session = self.select_previous(state)
                    }
                    KeyCode::Char('g') => state.selected_session = self.select_first(state),
                    KeyCode::Char('M') => state.selected_session = self.select_middle(state),
                    KeyCode::Char('G') => state.selected_session = self.select_last(state),
                    KeyCode::Char('/') => self.mode = MenuMode::SearchInsert,
                    KeyCode::Esc => self.search_bar = TextArea::default(),

                    // Mode switching
                    KeyCode::Char('a') => state.mode = AppMode::Create,
                    KeyCode::Char('r') => state.mode = AppMode::Rename,
                    KeyCode::Char('d') => state.mode = AppMode::Delete,
                    KeyCode::Tab => state.mode = AppMode::Presets,

                    // Control
                    KeyCode::Char('q') => state.exit = true,
                    KeyCode::Enter => {
                        // Get the locally selected index
                        // (since session menu may be applying a filter)
                        if let Some(local_selected_index) = self.list_state.selected() {
                            // Convert that to a global index, which indexes into the global array
                            // of tmux sessions
                            let global_selected_index =
                                self.displayed_sessions[local_selected_index];
                            if let Err(msg) =
                                tmux::switch_session(&state.sessions[global_selected_index].name)
                            {
                                send_timed_notification(&state.event_handler, msg);
                            }
                        };
                    }
                    _ => {}
                },
                MenuMode::SearchInsert => match key_event.code {
                    KeyCode::Enter => {
                        self.mode = MenuMode::Normal;
                        state.selected_session = self.select_first(state);
                    }
                    KeyCode::Esc => {
                        // Empty the search bar and reset displayed sessions
                        self.search_bar = TextArea::default();
                        self.mode = MenuMode::Normal;
                    }
                    _ => {
                        self.search_bar.input(key_event);
                    }
                },
            },
            AppEvent::ShowNotification(msg) => self.notification = Some(msg),
            AppEvent::ClearNotification => self.notification = None,
            _ => {}
        }
    }
}
