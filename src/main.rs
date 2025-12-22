use anyhow::Error;

use crate::app::App;
use rat_salsa::{RunConfig, SalsaAppContext, SalsaContext, poll::{PollCrossterm, PollRendered, PollTasks, PollTimers}, run_tui};
use std::io;

mod app;
mod render;
mod tmux;

pub enum AppEvent {}

pub struct Config {}

#[derive(Default)]
pub struct State {}

pub struct Global {
    ctx: SalsaAppContext<AppEvent, Error>,

    pub cfg: Config,
}

fn main() -> Result<(), Error> {
    let rt = tokio::runtime::Runtime::new()?;

    let config = Config::default();
    let mut global = Global::new(config);
    let mut state = State::default();

    let mut terminal = ratatui::init();
    let app = App::new();

    run_tui(
        init,
        render,
        App::handle_events,
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

impl SalsaContext<AppEvent, Error> for Global {
    fn set_salsa_ctx(&mut self, app_ctx: SalsaAppContext<AppEvent, Error>) {
        self.ctx = app_ctx;
    }

    #[inline(always)]
    fn salsa_ctx(&self) -> &SalsaAppContext<AppEvent, Error> {
        &self.ctx
    }
}

impl Global {
    pub fn new(cfg: Config) -> Self {
        Self {
            ctx: Default::default(),
            cfg,
        }
    }
}
