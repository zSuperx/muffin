use regex::Regex;
use std::process::Command;

use crate::app::Session;

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

    sessions.sort_by_key(|a| !a.active);
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
    // run_command("tmux", &["kill-session", "-t", target]).map(|_| ())
    Err("Some notification".to_string())
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
    use crate::tmux;

    #[test]
    fn test_list_session() {
        let x = tmux::list_sessions();

        println!("{:#?}", x);
    }

    #[test]
    fn test_create_delete_session() {
        let x = tmux::create_session("test_session");
        println!("{:#?}", x);

        let x = tmux::delete_session("test_session");
        println!("{:#?}", x);
    }
}
