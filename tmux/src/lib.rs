use regex::Regex;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct Session {
    pub name: String,
    pub windows: String,
    pub attached: bool,
    pub active: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum SplitDirection {
    Horizontal,
    Vertical,
}

#[derive(Clone, Debug)]
pub enum LayoutNode {
    Pane {
        cwd: String,
        command: Option<String>,
        size: u8,
    },
    Split {
        direction: SplitDirection,
        children: Vec<LayoutNode>,
        size: u8,
    },
}

impl LayoutNode {
    fn size(&self) -> u8 {
        match self {
            LayoutNode::Pane { size, .. } => *size,
            LayoutNode::Split { size, .. } => *size,
        }
    }
}

#[derive(Debug)]
pub struct Window {
    pub name: String,
    pub cwd: String,
    pub layout: LayoutNode,
}

#[derive(Debug)]
pub struct Preset {
    pub name: String,
    pub cwd: String,
    pub running: bool,
    pub windows: Vec<Window>,
}

pub fn spawn_preset(preset: &Preset) -> Result<(), String> {
    create_session(&preset.name)?;

    for (i, window_cfg) in preset.windows.iter().enumerate() {
        let window_target = if i == 0 {
            // Use the default window created by new-session
            run_command(
                "tmux",
                &[
                    "rename-window",
                    "-t",
                    &format!("{}:0", preset.name),
                    &window_cfg.name,
                ],
            )?;
            format!("{}:{}", preset.name, window_cfg.name)
        } else {
            // Create a new window and get its name/index
            run_command(
                "tmux",
                &[
                    "new-window",
                    "-t",
                    &preset.name,
                    "-n",
                    &window_cfg.name,
                    "-P",
                ],
            )?
            .trim()
            .to_string();
            format!("{}:{}", preset.name, window_cfg.name)
        };

        // Initial pane in a new window is always index 0
        let initial_pane = format!("{}.0", window_target);
        apply_layout_recursive(&initial_pane, &window_cfg.layout)?;
    }

    Ok(())
}

fn apply_layout_recursive(pane_target: &str, node: &LayoutNode) -> Result<(), String> {
    match node {
        LayoutNode::Pane { cwd, command, .. } => {
            run_command(
                "tmux",
                &[
                    "send-keys",
                    "-t",
                    pane_target,
                    &format!("cd {}", cwd),
                    "Enter",
                ],
            )?;
            // run command if provided
            if let Some(cmd) = command {
                run_command("tmux", &["send-keys", "-t", pane_target, cmd, "Enter"])?;
            }
            Ok(())
        }
        LayoutNode::Split {
            direction,
            children,
            ..
        } => {
            let mut current_pane_target = pane_target.to_string();
            let mut remaining_pct: f32 = children.iter().map(|c| c.size() as f32).sum();

            for (i, child) in children.iter().enumerate() {
                // If it's the last child, we don't split anymore;
                // it just occupies whatever is left in current_pane_target
                if i == children.len() - 1 {
                    apply_layout_recursive(&current_pane_target, child)?;
                    break;
                }

                let child_pct = child.size() as f32;

                // Warning: Borrowed from AI slop for math calculations

                // MATH CALCULATION:
                // Tmux '-p' is the size of the NEW pane relative to the target.
                // If child needs 20% of the current area, the NEW pane (the rest)
                // needs to be 80% of the current target.
                let split_p = (((remaining_pct - child_pct) / remaining_pct) * 100.0).round() as u8;

                // Split the window.
                // The 'old' index stays as the 'child', the 'new' index is the 'rest'.
                let (sess, win, new_index) =
                    split_window(&current_pane_target, split_p, direction)?;

                let next_pane_target = format!("{}:{}.{}", sess, win, new_index);

                // Recurse into the child we just "carved out"
                apply_layout_recursive(&current_pane_target, child)?;

                // Move our focus to the newly created pane for the next iteration
                current_pane_target = next_pane_target;
                remaining_pct -= child_pct;
            }
            Ok(())
        }
    }
}

pub fn split_window(
    target: &str,
    size: u8,
    direction: &SplitDirection,
) -> Result<(String, String, usize), String> {
    let direction_flag = match direction {
        SplitDirection::Horizontal => "-h",
        SplitDirection::Vertical => "-v",
    };
    let output = run_command(
        "tmux",
        &[
            "split-window",
            "-t",
            target,
            direction_flag,
            "-p",
            size.to_string().as_str(),
            "-P",
        ],
    )?;
    let (session_name, rest) = output.trim().split_once(":").ok_or("Unexpected output")?;
    let (window_name, pane_index) = rest.split_once(".").ok_or("Unexpected output")?;
    Ok((
        session_name.into(),
        window_name.into(),
        pane_index.parse::<usize>().map_err(|_| "Parsing error")?,
    ))
}

pub fn list_sessions() -> Result<Vec<Session>, String> {
    let output = run_command("tmux", &["list-sessions"])?;
    let active_session_name = match std::env::var("TMUX_PANE") {
        Ok(tmux_pane_env) => Some(
            run_command(
                "tmux",
                &["display-message", "-t", &tmux_pane_env, "-p", "'#S'"],
            )?
            .trim()
            .trim_matches('\'')
            .to_string(),
        ),
        Err(_) => None,
    };

    let active_regex = Regex::new(r"\(attached\)$").unwrap();
    let windows_regex = Regex::new(r"^(.+?): (\d+).*").unwrap();

    let sessions = output
        .lines()
        .map(|line| {
            let captures = windows_regex.captures(line).unwrap();

            let name = Some(captures[1].to_string());

            Session {
                windows: captures[2].to_string(),
                attached: active_regex.is_match(line),
                active: name == active_session_name,
                name: name.unwrap(),
            }
        })
        .collect::<Vec<Session>>();

    Ok(sessions)
}

pub fn switch_session(target: &str) -> Result<(), String> {
    run_command("tmux", &["switch-client", "-t", target]).map(|_| ())
}

pub fn create_session(new_name: &str) -> Result<(), String> {
    if new_name.is_empty() {
        run_command("tmux", &["new-session", "-d"]).map(|_| ())
    } else {
        run_command("tmux", &["new-session", "-s", new_name, "-d"]).map(|_| ())
    }
}

pub fn rename_session(target: &str, new_name: &str) -> Result<(), String> {
    run_command("tmux", &["rename-session", "-t", target, new_name]).map(|_| ())
}

pub fn delete_session(target: &str) -> Result<(), String> {
    run_command("tmux", &["kill-session", "-t", target]).map(|_| ())
}

fn run_command(command: &str, args: &[&str]) -> Result<String, String> {
    let output = Command::new(command)
        .args(args)
        .output()
        .map_err(|_| "Error running command")?;

    if output.status.code().is_none_or(|code| code != 0) {
        return Err(String::from_utf8(output.stderr).map_err(|_| "Error decoding output")?);
    }

    Ok(String::from_utf8(output.stdout).map_err(|_| "Error decoding output")?)
}
