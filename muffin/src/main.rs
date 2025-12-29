use app::driver::App;
mod app;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), String> {
    let mut args = std::env::args();
    let arg0 = args.next().unwrap();

    let mut presets_path = "~/.config/muffin/presets.kdl".to_string();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--presets" | "-p" => {
                presets_path = args.next().ok_or(format!("{arg} expects a path"))?;
            }
            "--help" | "-h" => {
                eprintln!(
                    r"
Usage: {arg0} [OPTIONS]

OPTIONS:
    -p, --presets <FILE>    Path to KDL file with session presets
    -h, --help              Print help
                        ",
                );
                std::process::exit(1);
            }
            x => {
                eprintln!("Unknown flag or value '{x}'. Run '{arg0} --help' for usage.",);
                std::process::exit(1);
            }
        }
    }

    let presets_path = shellexpand::full(&presets_path)
        .expect("Failed to expand environment variables in path")
        .to_string();

    let sessions = tmux::list_sessions()?;
    let presets_str: String = std::fs::read(&presets_path)
        .expect("Error reading file.")
        .try_into()
        .expect("Error parsing file into a string.");

    let presets = parser::parse_config(&presets_str)?;

    let mut app = App::new(sessions, presets, presets_path.to_string());

    let mut terminal = ratatui::init();
    let app_result = app.run(&mut terminal).await;

    ratatui::restore();
    app_result
}
