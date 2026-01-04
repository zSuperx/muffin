use app::driver::App;
use tmux::Preset;
mod app;

#[tokio::main(flavor = "current_thread")]
async fn main() -> () {
    let mut args = std::env::args();
    let arg0 = args.next().unwrap();
    let mut list_presets = false;
    let mut start_preset = None;
    let mut custom_preset = None;
    let mut exit_on_switch = false;
    let dot_config_muffin = shellexpand::full("~/.config/muffin").unwrap().to_string();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--presets" | "-p" => {
                custom_preset = Some(args.next().unwrap_or_else(|| {
                    eprintln!("Error: {arg} expects a path");
                    std::process::exit(1);
                }));
            }
            "--help" | "-h" => {
                print_help(&arg0);
                std::process::exit(1);
            }
            "--list-presets" | "-l" => {
                list_presets = true;
            }
            "--start-preset" | "-s" => {
                start_preset = Some(args.next().unwrap_or_else(|| {
                    eprintln!("Error: {arg} expects a preset name");
                    std::process::exit(1);
                }));
            }
            "--exit-on-switch" | "-e" => {
                exit_on_switch = true;
            }
            x => {
                eprintln!("Unknown flag or value '{x}'. Run '{arg0} --help' for usage.");
                std::process::exit(1);
            }
        }
    }

    let presets_str = match custom_preset {
        Some(s) => {
            let presets_path = shellexpand::full(&s)
                .expect("Failed to expand environment variables in path")
                .to_string();
            std::fs::read(&presets_path)
                .ok()
                .and_then(|x| String::from_utf8(x).ok())
                .unwrap_or_else(|| {
                    eprintln!("Could not read from'{presets_path}'");
                    std::process::exit(1);
                })
        }
        None => {
            if std::fs::exists(format!("{dot_config_muffin}/presets.kdl")).unwrap() {
                std::fs::read(format!("{dot_config_muffin}/presets.kdl"))
                    .ok()
                    .and_then(|x| String::from_utf8(x).ok())
                    .unwrap_or_else(|| {
                        eprintln!(
                            "Could not open path '{dot_config_muffin}/presets.kdl'. Does it exist?"
                        );
                        std::process::exit(1);
                    })
            } else {
                std::fs::create_dir_all(format!("{dot_config_muffin}")).unwrap();
                std::fs::write(
                    format!("{dot_config_muffin}/presets.kdl"),
                    EXAMPLE_PRESET_CONTENT,
                )
                .unwrap();
                EXAMPLE_PRESET_CONTENT.into()
            }
        }
    };

    let presets = parser::parse_config(&presets_str).unwrap_or_else(|_| {
        eprintln!("Failed to parse configuration file.");
        std::process::exit(1);
    });

    if list_presets {
        for Preset {
            name, cwd, windows, ..
        } in presets.values()
        {
            println!("Session: {name}, {} windows, cwd: {cwd}", windows.len());
        }
        return;
    }

    if let Some(preset_name) = start_preset {
        let preset_to_start = presets.get(&preset_name).unwrap_or_else(|| {
            eprintln!("Preset does not exist!");
            std::process::exit(1);
        });
        tmux::spawn_preset(preset_to_start).unwrap_or_else(|e| {
            eprintln!("{e}");
            std::process::exit(1);
        });
        tmux::switch_session(&preset_to_start.name).unwrap_or_else(|e| {
            eprintln!("{e}");
            std::process::exit(1);
        });
        return;
    }

    let sessions = tmux::list_sessions().unwrap_or_else(|e| {
        eprintln!("{e}");
        std::process::exit(1);
    });
    let mut app = App::new(sessions, presets, dot_config_muffin.to_string(), exit_on_switch);

    let mut terminal = ratatui::init();
    let app_result = app.run(&mut terminal).await;

    ratatui::restore();
    app_result.unwrap();
}

fn print_help(arg0: &str) {
    eprintln!(
        r"
Usage: {arg0} [OPTIONS]

OPTIONS:
    -s, --start-preset <NAME>   Start preset
    -l, --list-presets          List presets information
    -p, --presets <FILE>        Path to presets file [default: ~/.config/muffin/presets.kdl]
    -h, --help                  Print help",
    );
}

const EXAMPLE_PRESET_CONTENT: &'static str = r#"
session name="foo" cwd="~" {
  window {
    split direction="h" {
      pane command="echo Hello,"
      pane command="echo World!"
    }
  }
}
"#;
