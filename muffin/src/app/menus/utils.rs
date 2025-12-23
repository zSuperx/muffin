use std::time::Duration;

use ratatui::{
    layout::{Constraint, Flex, Layout, Rect},
    style::Stylize,
    text::{Line, Span},
};

use crate::app::app::{AppEvent, EventHandler, Mode};

#[allow(unused)]
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

pub fn make_instructions<'a>(instructions: Vec<(&'a str, &'a str)>) -> Line<'a> {
    Line::from(
        instructions
            .iter()
            .flat_map(|(key, desc)| {
                vec![format!(" {}", key).gray(), format!(":{desc} ").dark_gray()]
            })
            .collect::<Vec<Span>>(),
    )
}


pub fn send_timed_notification(event_handler: &EventHandler, msg: String) {
    let tx = event_handler.tx.clone();

    // Immediately show notification
    let _ = tx.send(AppEvent::ShowNotification(msg));

    // Spawn a background task to clear it after 3 seconds
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(3)).await;
        let _ = tx.send(AppEvent::ClearNotification);
    });
}
