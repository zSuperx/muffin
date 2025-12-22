#![allow(unused, dead_code)]
use anyhow::Error;
use ratatui::widgets::ListState;

use crate::app::App;
use rat_salsa::{RunConfig, SalsaAppContext, SalsaContext, poll::{PollCrossterm, PollRendered, PollTasks, PollTimers}, run_tui};
use std::io;

mod app;
mod render;
mod tmux;

fn main() -> Result<(), Error> {
    let rt = tokio::runtime::Runtime::new()?;

    let config = Config::default();
    let mut global = Global::new(config);
    let mut state = State::default();

    let mut terminal = ratatui::init();
    let app = App::new();

    run_tui(
        app::init,
        render::render,
        app::event,
        error,
        &mut global,
        &mut state,
        RunConfig::default()?
            .poll(PollCrossterm)
            .poll(PollTimers::default())
            .poll(PollTasks::default())
            .poll(PollRendered)
            .poll(rat_salsa::poll::PollTokio::new(rt)),
    )?;
}
