use crate::process::command::new_command;

pub fn build_taskkill_args(pid: u32) -> Vec<String> {
    vec![
        "/F".to_string(),
        "/T".to_string(),
        "/PID".to_string(),
        pid.to_string(),
    ]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KillTreeOutcome {
    Killed,
    AlreadyExited,
}

pub fn kill_process_tree(pid: u32) -> Result<KillTreeOutcome, String> {
    let output = new_command("taskkill")
        .args(build_taskkill_args(pid))
        .output()
        .map_err(|error| error.to_string())?;

    if output.status.success() {
        return Ok(KillTreeOutcome::Killed);
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let message = format!("{stdout}\n{stderr}");
    if is_process_not_found_message(&message) {
        return Ok(KillTreeOutcome::AlreadyExited);
    }

    Err(message.trim().to_lowercase())
}

fn is_process_not_found_message(message: &str) -> bool {
    let normalized = message.to_lowercase();
    normalized.contains("not found")
        || normalized.contains("no running instance")
        || normalized.contains("\u{627e}\u{4e0d}\u{5230}")
        || normalized.contains("\u{6ca1}\u{6709}\u{627e}\u{5230}")
}

#[cfg(test)]
mod tests {
    use super::{build_taskkill_args, is_process_not_found_message};

    #[test]
    fn builds_windows_process_tree_kill_args() {
        assert_eq!(
            build_taskkill_args(4321),
            vec![
                "/F".to_string(),
                "/T".to_string(),
                "/PID".to_string(),
                "4321".to_string()
            ]
        );
    }

    #[test]
    fn treats_taskkill_process_not_found_as_already_exited() {
        assert!(is_process_not_found_message(
            "ERROR: The process \"4321\" not found."
        ));
        assert!(is_process_not_found_message(
            "\u{9519}\u{8bef}: \u{627e}\u{4e0d}\u{5230}\u{8fdb}\u{7a0b} \"4321\"."
        ));
    }
}
