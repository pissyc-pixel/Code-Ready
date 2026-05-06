use std::{
    fs::{self, File},
    io::{BufRead, BufReader, Write},
    path::PathBuf,
    process::Stdio,
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

use tauri::{AppHandle, Emitter, Manager};

use crate::{
    config::AppConfig,
    detector::ToolInstallStatus,
    detector::{self, DetectResultEvent},
    installer::{
        ccswitch, claude, codex, git, node, now_timestamp, opencode, python, InstallPhase,
        InstallProgressEvent, InstallRequestMode, InstallResult, InstallStatusEvent,
    },
    logger::redact::redact_line,
    process::command::new_command,
    process::kill_tree::{kill_process_tree, KillTreeOutcome},
    state::AppState,
};

#[derive(Debug, Clone)]
pub struct InstallCommandSpec {
    pub program: String,
    pub args: Vec<String>,
    pub envs: Vec<(String, String)>,
    pub timeout: Duration,
    pub started_suggestion: Option<String>,
    pub success_suggestion: Option<String>,
    pub failure_suggestion: Option<String>,
    pub detect_after: Vec<String>,
}

#[allow(dead_code)]
pub fn build_command_spec(tool_id: &str, config: &AppConfig) -> Result<InstallCommandSpec, String> {
    build_command_spec_for_mode(tool_id, config, InstallRequestMode::Install)
}

pub fn build_command_spec_for_mode(
    tool_id: &str,
    config: &AppConfig,
    mode: InstallRequestMode,
) -> Result<InstallCommandSpec, String> {
    match tool_id {
        "ccswitch" => {
            if matches!(mode, InstallRequestMode::Latest) {
                Err("latest install is not available for ccswitch in V0.5".to_string())
            } else {
                ccswitch::command_spec(config)
            }
        }
        "claude" => claude::command_spec_for_mode(config, mode),
        "codex" => codex::command_spec_for_mode(config, mode),
        "git" => {
            if matches!(mode, InstallRequestMode::Latest) {
                Err("latest install is not available for git in V0.5".to_string())
            } else {
                Ok(git::command_spec(config))
            }
        }
        "node" => {
            if matches!(mode, InstallRequestMode::Latest) {
                Err("latest install is not available for node in V0.5".to_string())
            } else {
                Ok(node::command_spec(config))
            }
        }
        "opencode" => opencode::command_spec_for_mode(config, mode),
        "python" => {
            if matches!(mode, InstallRequestMode::Latest) {
                Err("latest install is not available for python in V0.5".to_string())
            } else {
                Ok(python::command_spec(config))
            }
        }
        "__test_success__" => Ok(InstallCommandSpec {
            program: "powershell".to_string(),
            args: vec![
                "-NoProfile".to_string(),
                "-Command".to_string(),
                "Write-Output 'token=abc123'; Start-Sleep -Milliseconds 100; Write-Output 'Authorization: Bearer secret-token'; Start-Sleep -Milliseconds 100".to_string(),
            ],
            envs: Vec::new(),
            timeout: Duration::from_secs(10),
            started_suggestion: None,
            success_suggestion: Some(
                "V0.3 commit 3 宸插畬鎴愬悗鍙板畨瑁呴摼璺獙璇侊紝鐪熷疄瀹夎鍣ㄥ皢鍦ㄤ笅涓€涓彁浜や腑鎺ュ叆銆?"
                    .to_string(),
            ),
            failure_suggestion: None,
            detect_after: Vec::new(),
        }),
        "__test_timeout__" => Ok(InstallCommandSpec {
            program: "powershell".to_string(),
            args: vec![
                "-NoProfile".to_string(),
                "-Command".to_string(),
                "Start-Sleep -Seconds 3".to_string(),
            ],
            envs: Vec::new(),
            timeout: Duration::from_millis(500),
            started_suggestion: None,
            success_suggestion: None,
            failure_suggestion: Some("install task timed out".to_string()),
            detect_after: Vec::new(),
        }),
        "__test_cancel__" => Ok(InstallCommandSpec {
            program: "powershell".to_string(),
            args: vec![
                "-NoProfile".to_string(),
                "-Command".to_string(),
                "1..20 | ForEach-Object { Write-Output \"line=$_\"; Start-Sleep -Milliseconds 200 }".to_string(),
            ],
            envs: Vec::new(),
            timeout: Duration::from_secs(10),
            started_suggestion: None,
            success_suggestion: None,
            failure_suggestion: None,
            detect_after: Vec::new(),
        }),
        _ => Err(format!(
            "installer for {tool_id} is not available in the current version"
        )),
    }
}

pub fn spawn_install_runner(app: AppHandle, tool_id: String, spec: InstallCommandSpec) {
    thread::spawn(move || {
        if let Err(error) = run_install_task(app.clone(), tool_id.clone(), spec) {
            let _ = app.state::<AppState>().clear_install(&tool_id);
            let _ = app.emit(
                "install:status",
                InstallStatusEvent {
                    tool_id,
                    status: ToolInstallStatus::InstallFailed,
                    phase: InstallPhase::Failed,
                    error_message: Some(error),
                    suggestion: None,
                    result: None,
                    timestamp: now_timestamp(),
                },
            );
        }
    });
}

fn run_install_task(
    app: AppHandle,
    tool_id: String,
    spec: InstallCommandSpec,
) -> Result<(), String> {
    let state = app.state::<AppState>();
    let logger = TaskLogger::new(&tool_id)?;
    let started_at = Instant::now();

    let mut child = new_command(&spec.program)
        .args(&spec.args)
        .envs(spec.envs.iter().cloned())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| error.to_string())?;

    let pid = child.id();
    state.update_install_pid(&tool_id, pid)?;
    emit_status(
        &app,
        InstallStatusEvent {
            tool_id: tool_id.clone(),
            status: ToolInstallStatus::Installing,
            phase: InstallPhase::Started,
            error_message: None,
            suggestion: spec.started_suggestion.clone(),
            result: None,
            timestamp: now_timestamp(),
        },
    )?;
    state.update_install_phase(&tool_id, InstallPhase::Running)?;
    emit_status(&app, running_status_event(&tool_id))?;

    let stdout_handle = child.stdout.take().map(|stdout| {
        spawn_stream_reader(
            app.clone(),
            logger.clone(),
            tool_id.clone(),
            "stdout",
            stdout,
        )
    });
    let stderr_handle = child.stderr.take().map(|stderr| {
        spawn_stream_reader(
            app.clone(),
            logger.clone(),
            tool_id.clone(),
            "stderr",
            stderr,
        )
    });

    if state.is_cancel_requested(&tool_id)? {
        let _ = kill_process_tree(pid);
    }

    let mut cancel_effective = false;
    let mut timed_out = false;

    let exit_code = loop {
        if started_at.elapsed() >= spec.timeout {
            timed_out = true;
            let _ = kill_process_tree(pid);
        }

        if state.is_cancel_requested(&tool_id)? {
            match kill_process_tree(pid) {
                Ok(KillTreeOutcome::Killed) => {
                    cancel_effective = true;
                }
                Ok(KillTreeOutcome::AlreadyExited) | Err(_) => {
                    // Process already exited, or taskkill returned an unexpected error.
                    // Either way the process is gone or unreachable; the poll loop will
                    // observe the natural exit and cancel_requested drives the final
                    // Cancelled status rather than Failed.
                }
            }
        }

        match child.try_wait().map_err(|error| error.to_string())? {
            Some(status) => break status.code(),
            None => thread::sleep(Duration::from_millis(100)),
        }
    };

    if let Some(handle) = stdout_handle {
        let _ = handle.join();
    }
    if let Some(handle) = stderr_handle {
        let _ = handle.join();
    }

    let duration_ms = started_at.elapsed().as_millis() as u64;
    let cancel_requested = state.is_cancel_requested(&tool_id)?;
    let final_event = choose_final_status_event(
        &tool_id,
        exit_code,
        timed_out,
        cancel_requested,
        cancel_effective,
        duration_ms,
        spec.success_suggestion.clone(),
        spec.failure_suggestion.clone(),
    );

    state.clear_install(&tool_id)?;
    emit_status(&app, final_event)?;
    if exit_code == Some(0) && !timed_out && !cancel_requested && !cancel_effective {
        let _ = emit_detection_results(&app, &spec.detect_after);
    }
    Ok(())
}

fn choose_final_status_event(
    tool_id: &str,
    exit_code: Option<i32>,
    timed_out: bool,
    cancel_requested: bool,
    cancel_effective: bool,
    duration_ms: u64,
    success_suggestion: Option<String>,
    failure_suggestion: Option<String>,
) -> InstallStatusEvent {
    let result = Some(InstallResult {
        exit_code,
        duration_ms: Some(duration_ms),
    });

    if cancel_requested || cancel_effective {
        return InstallStatusEvent {
            tool_id: tool_id.to_string(),
            status: ToolInstallStatus::Missing,
            phase: InstallPhase::Cancelled,
            error_message: None,
            suggestion: Some("Install was cancelled. Run install again when ready.".to_string()),
            result,
            timestamp: now_timestamp(),
        };
    }

    if timed_out {
        return InstallStatusEvent {
            tool_id: tool_id.to_string(),
            status: ToolInstallStatus::InstallFailed,
            phase: InstallPhase::Timeout,
            error_message: Some("install task timed out".to_string()),
            suggestion: failure_suggestion,
            result,
            timestamp: now_timestamp(),
        };
    }

    if exit_code == Some(0) {
        return InstallStatusEvent {
            tool_id: tool_id.to_string(),
            status: ToolInstallStatus::Installed,
            phase: InstallPhase::Success,
            error_message: None,
            suggestion: success_suggestion,
            result,
            timestamp: now_timestamp(),
        };
    }

    InstallStatusEvent {
        tool_id: tool_id.to_string(),
        status: ToolInstallStatus::InstallFailed,
        phase: InstallPhase::Failed,
        error_message: Some("install task exited with a non-zero code".to_string()),
        suggestion: failure_suggestion,
        result,
        timestamp: now_timestamp(),
    }
}

fn emit_status(app: &AppHandle, event: InstallStatusEvent) -> Result<(), String> {
    app.emit("install:status", event)
        .map_err(|error| error.to_string())
}

fn running_status_event(tool_id: &str) -> InstallStatusEvent {
    InstallStatusEvent {
        tool_id: tool_id.to_string(),
        status: ToolInstallStatus::Installing,
        phase: InstallPhase::Running,
        error_message: None,
        suggestion: None,
        result: None,
        timestamp: now_timestamp(),
    }
}

fn spawn_stream_reader<R>(
    app: AppHandle,
    logger: TaskLogger,
    tool_id: String,
    stream: &'static str,
    reader: R,
) -> thread::JoinHandle<()>
where
    R: std::io::Read + Send + 'static,
{
    thread::spawn(move || {
        let buffered = BufReader::new(reader);
        for line in buffered.lines().map_while(Result::ok) {
            let redacted = redact_line(&line);
            let _ = app.emit(
                "install:progress",
                InstallProgressEvent {
                    tool_id: tool_id.clone(),
                    phase: InstallPhase::Running,
                    line: redacted.clone(),
                    stream: stream.to_string(),
                    timestamp: now_timestamp(),
                },
            );
            let _ = logger.write_line(stream, &redacted);
        }
    })
}

#[derive(Clone)]
struct TaskLogger {
    file: Arc<Mutex<File>>,
}

impl TaskLogger {
    fn new(tool_id: &str) -> Result<Self, String> {
        let path = log_path(tool_id)?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        let file = File::create(path).map_err(|error| error.to_string())?;
        Ok(Self {
            file: Arc::new(Mutex::new(file)),
        })
    }

    fn write_line(&self, stream: &str, line: &str) -> Result<(), String> {
        let mut file = self
            .file
            .lock()
            .map_err(|_| "install log poisoned".to_string())?;
        writeln!(file, "[{stream}] {line}").map_err(|error| error.to_string())
    }
}

fn log_path(tool_id: &str) -> Result<PathBuf, String> {
    let appdata = std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .ok_or_else(|| "APPDATA is not available".to_string())?;
    Ok(appdata
        .join("ai-coding-installer")
        .join("logs")
        .join(format!(
            "{tool_id}-{}.log",
            chrono::Local::now().format("%Y%m%d-%H%M%S")
        )))
}

fn emit_detection_results(app: &AppHandle, tool_ids: &[String]) -> Result<(), String> {
    for tool_id in tool_ids {
        let result = tauri::async_runtime::block_on(detector::detect_tool(app.clone(), tool_id))?;
        app.emit(
            "detect:result",
            DetectResultEvent {
                tool_id: tool_id.clone(),
                result,
            },
        )
        .map_err(|error| error.to_string())?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{build_command_spec, choose_final_status_event, running_status_event};
    use crate::config::AppConfig;
    use crate::installer::InstallPhase;

    #[test]
    fn supports_test_runner_specs() {
        let spec = build_command_spec("__test_success__", &AppConfig::default())
            .expect("support test command");
        assert_eq!(spec.program, "powershell");
        assert!(!spec.args.is_empty());
    }

    #[test]
    fn running_status_event_marks_tool_as_installing() {
        let event = running_status_event("git");

        assert_eq!(event.tool_id, "git");
        assert_eq!(event.phase, InstallPhase::Running);
        assert!(matches!(
            event.status,
            crate::detector::ToolInstallStatus::Installing
        ));
        assert!(event.result.is_none());
    }

    #[test]
    fn final_status_prefers_cancelled_when_cancel_requested_races_with_success_exit() {
        let event = choose_final_status_event("git", Some(0), false, true, false, 123, None, None);

        assert_eq!(event.phase, InstallPhase::Cancelled);
        assert!(matches!(
            event.status,
            crate::detector::ToolInstallStatus::Missing
        ));
        assert!(event.error_message.is_none());
    }

    #[test]
    fn final_status_reports_timeout_before_regular_failure() {
        let event = choose_final_status_event(
            "git",
            None,
            true,
            false,
            false,
            123,
            None,
            Some("install task timed out".to_string()),
        );

        assert_eq!(event.phase, InstallPhase::Timeout);
        assert!(matches!(
            event.status,
            crate::detector::ToolInstallStatus::InstallFailed
        ));
        assert_eq!(
            event.error_message.as_deref(),
            Some("install task timed out")
        );
    }

    #[test]
    fn final_status_reports_failed_for_non_zero_exit() {
        let event = choose_final_status_event(
            "git",
            Some(1),
            false,
            false,
            false,
            123,
            None,
            Some("install failed".to_string()),
        );

        assert_eq!(event.phase, InstallPhase::Failed);
        assert!(matches!(
            event.status,
            crate::detector::ToolInstallStatus::InstallFailed
        ));
        assert_eq!(
            event.error_message.as_deref(),
            Some("install task exited with a non-zero code")
        );
    }

    #[test]
    fn final_status_is_cancelled_when_kill_failed_but_cancel_was_requested() {
        // cancel_effective=false means taskkill returned an error (process unreachable),
        // but cancel_requested=true means the user did request cancellation.
        // The final status must still be Cancelled, not Failed.
        let event =
            choose_final_status_event("claude", Some(1), false, true, false, 200, None, None);

        assert_eq!(event.phase, InstallPhase::Cancelled);
        assert!(matches!(
            event.status,
            crate::detector::ToolInstallStatus::Missing
        ));
        assert!(event.error_message.is_none());
    }

    #[test]
    fn success_path_is_unaffected_when_cancel_was_never_requested() {
        let event = choose_final_status_event(
            "codex",
            Some(0),
            false,
            false,
            false,
            500,
            Some("done".to_string()),
            None,
        );

        assert_eq!(event.phase, InstallPhase::Success);
        assert!(matches!(
            event.status,
            crate::detector::ToolInstallStatus::Installed
        ));
        assert_eq!(event.suggestion.as_deref(), Some("done"));
    }
}
