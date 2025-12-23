use app::app::App;
use std::io;

mod app;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), ()> {
    let mut terminal = ratatui::init();
    let app_result = App::new().run(&mut terminal).await;
    ratatui::restore();
    app_result
}

