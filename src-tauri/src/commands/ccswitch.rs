use std::{
    path::{Path, PathBuf},
    process::Command,
};

use crate::{config, detector};

#[tauri::command]
pub async fn open_ccswitch() -> Result<(), String> {
    let config = config::get_config()?;
    let configured_path = config.ccswitch_path.map(PathBuf::from);
    let executable = detector::resolve_ccswitch_path(configured_path)
        .ok_or_else(|| "ccSwitch executable path is not configured or detected".to_string())?;

    validate_ccswitch_executable(&executable)?;

    Command::new(&executable)
        .spawn()
        .map_err(|error| format!("failed to open ccSwitch: {error}"))?;

    Ok(())
}

fn validate_ccswitch_executable(path: &Path) -> Result<(), String> {
    if !path.exists() {
        return Err(format!("ccSwitch executable was not found at {}", path.display()));
    }

    let is_exe = path
        .extension()
        .and_then(|item| item.to_str())
        .map(|item| item.eq_ignore_ascii_case("exe"))
        .unwrap_or(false);

    if !is_exe {
        return Err("ccSwitch path must point to a .exe file".to_string());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::validate_ccswitch_executable;
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    fn temp_test_path(file_name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time")
            .as_nanos();
        std::env::temp_dir().join(format!("aicoding-{unique}-{file_name}"))
    }

    #[test]
    fn rejects_missing_ccswitch_path() {
        let error = validate_ccswitch_executable(PathBuf::from("C:\\missing\\ccswitch.exe").as_path())
            .expect_err("missing path should fail");
        assert!(error.contains("not found"));
    }

    #[test]
    fn rejects_non_executable_ccswitch_path() {
        let path = temp_test_path("ccswitch.txt");
        fs::write(&path, "placeholder").expect("write temp file");

        let error = validate_ccswitch_executable(&path).expect_err("non-exe should fail");
        assert!(error.contains(".exe"));

        let _ = fs::remove_file(path);
    }

    #[test]
    fn accepts_existing_executable_path() {
        let path = temp_test_path("ccswitch.exe");
        fs::write(&path, "placeholder").expect("write temp file");

        validate_ccswitch_executable(&path).expect("exe path should pass");

        let _ = fs::remove_file(path);
    }
}
