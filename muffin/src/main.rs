use app::app::App;
use std::io;

mod app;

#[tokio::main(flavor = "current_thread")]
async fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let app_result = App::default().run(&mut terminal).await;
    ratatui::restore();
    app_result
}

