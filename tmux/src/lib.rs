use regex::Regex;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct Session {
    pub name: String,
    pub windows: String,
    pub active: bool,
}

#[derive(Clone, Copy)]
pub enum SplitDirection {
    Horizontal,
    Vertical,
}

#[derive(Clone)]
pub enum LayoutNode {
    Pane {
        cwd: Option<String>,
        command: Option<String>,
        percentage: u8,
    },
    Split {
        direction: SplitDirection,
        children: Vec<LayoutNode>,
        percentage: u8,
    },
}

impl LayoutNode {
    fn percentage(&self) -> u8 {
        match self {
            LayoutNode::Pane { percentage, .. } => *percentage,
            LayoutNode::Split { percentage, .. } => *percentage,
        }
    }
}

pub struct Window {
    pub name: String,
    pub layout: LayoutNode,
}

pub struct Preset {
    pub name: String,
    pub windows: Vec<Window>,
}

fn verify_layout_recursive(layout: &LayoutNode) -> Result<(), String> {
    if let LayoutNode::Split { children, .. } = layout {
        let percentages: Vec<_> = children.iter().map(|c| c.percentage()).collect();
        let sum: u8 = percentages.iter().sum();
        if sum != 100 {
            return Err(format!("Percentages {:?} add up to {}, expected 100.", percentages, sum));
        } else {
            return children.iter().map(verify_layout_recursive).collect();
        }
    }
    Ok(())
}

fn verify_preset(preset: &Preset) -> Result<(), String> {
    for window_cfg in preset.windows.iter() {
        verify_layout_recursive(&window_cfg.layout)?;
    }
    Ok(())
}

pub fn spawn_preset(preset: Preset) -> Result<(), String> {
    verify_preset(&preset)?;
    create_session(&preset.name)?;

    for (i, window_cfg) in preset.windows.into_iter().enumerate() {
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
        apply_layout_recursive(&initial_pane, window_cfg.layout)?;
    }

    Ok(())
}

fn apply_layout_recursive(pane_target: &str, node: LayoutNode) -> Result<(), String> {
    match node {
        LayoutNode::Pane { cwd, command, .. } => {
            // cd to cwd if provided
            if let Some(path) = cwd {
                run_command(
                    "tmux",
                    &[
                        "send-keys",
                        "-t",
                        pane_target,
                        &format!("cd {}", path),
                        "Enter",
                    ],
                )?;
            }
            // run command if provided
            if let Some(cmd) = command {
                run_command("tmux", &["send-keys", "-t", pane_target, &cmd, "Enter"])?;
            }
            Ok(())
        }
        LayoutNode::Split {
            direction,
            children,
            ..
        } => {
            let mut current_pane_target = pane_target.to_string();
            let mut remaining_pct: f32 = 100.0;

            for (i, child) in children.iter().enumerate() {
                // If it's the last child, we don't split anymore;
                // it just occupies whatever is left in current_pane_target
                if i == children.len() - 1 {
                    apply_layout_recursive(&current_pane_target, child.clone())?;
                    break;
                }

                let child_pct = child.percentage() as f32;

                // Warning: Borrowed from AI slop for math calculations

                // MATH CALCULATION:
                // Tmux '-p' is the percentage of the NEW pane relative to the target.
                // If child needs 20% of the current area, the NEW pane (the rest)
                // needs to be 80% of the current target.
                let split_p = (((remaining_pct - child_pct) / remaining_pct) * 100.0).round() as u8;

                // Split the window.
                // The 'old' index stays as the 'child', the 'new' index is the 'rest'.
                let (sess, win, new_index) =
                    split_window(&current_pane_target, split_p, direction)?;

                let next_pane_target = format!("{}:{}.{}", sess, win, new_index);

                // Recurse into the child we just "carved out"
                apply_layout_recursive(&current_pane_target, child.clone())?;

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
    percentage: u8,
    direction: SplitDirection,
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
            percentage.to_string().as_str(),
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

    let active_regex = Regex::new(r"\(attached\)$").unwrap();
    let windows_regex = Regex::new(r"^(.+?): (\d+).*").unwrap();

    let mut sessions = output
        .lines()
        .map(|line| {
            let captures = windows_regex.captures(line).unwrap();

            Session {
                name: captures[1].to_string(),
                windows: captures[2].to_string(),
                active: active_regex.is_match(line),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_session() {
        let x = list_sessions();

        println!("{:#?}", x);
    }

    #[test]
    fn test_create_delete_session() {
        let x = create_session("test_session");
        println!("{:#?}", x);

        let x = delete_session("test_session");
        println!("{:#?}", x);
    }

    #[test]
    fn test_split_window() {
        let x = split_window("muffin:BOBBY.0", 50, crate::SplitDirection::Horizontal).unwrap();
        println!("{:?}", x);
    }

    #[test]
    fn test_spawn_preset() {
        use SplitDirection::*;
        let layout1 = LayoutNode::Split {
            direction: Horizontal,
            children: vec![
                LayoutNode::Pane {
                    cwd: None,
                    command: None,
                    percentage: 33,
                },
                LayoutNode::Pane {
                    cwd: None,
                    command: Some("nvim".to_string()),
                    percentage: 34,
                },
                LayoutNode::Pane {
                    cwd: None,
                    command: None,
                    percentage: 33,
                },
            ],
            percentage: 100,
        };

        let layout2 = LayoutNode::Split {
            direction: Horizontal,
            children: vec![
                LayoutNode::Pane {
                    cwd: Some("~/zNix".to_string()),
                    command: Some("nvim".to_string()),
                    percentage: 50,
                },
                LayoutNode::Split {
                    direction: Vertical,
                    children: vec![
                        LayoutNode::Pane {
                            cwd: Some("~/zNix".to_string()),
                            command: Some("git status".to_string()),
                            percentage: 50,
                        },
                        LayoutNode::Pane {
                            cwd: Some("~/zNix".to_string()),
                            command: None,
                            percentage: 50,
                        },
                    ],
                    percentage: 50,
                },
            ],
            percentage: 100,
        };

        let window1 = Window {
            name: "BOBBY".into(),
            layout: layout1,
        };
        let window2 = Window {
            name: "BOBBY TWO".into(),
            layout: layout2,
        };

        let preset = Preset {
            name: "test-preset".into(),
            windows: vec![window1, window2],
        };

        spawn_preset(preset).unwrap();
    }
}
