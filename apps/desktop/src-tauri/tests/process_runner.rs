use std::ffi::OsString;
use std::path::Path;
use std::time::Duration;

use code_ready_desktop_lib::platform::process::{
    NativeProcessRunner, ProcessOutcome, ProcessRequest, ProcessRunner,
};

#[test]
fn probe_fixture_entrypoint() {
    match std::env::var("CODE_READY_PROCESS_FIXTURE").as_deref() {
        Ok("success") => print!("{}", "x".repeat(70 * 1024)),
        Ok("nonzero") => std::process::exit(23),
        Ok("timeout") => std::thread::sleep(Duration::from_secs(1)),
        _ => {}
    }
}

fn fixture_request(executable: &Path, fixture: &str, timeout: Duration) -> ProcessRequest {
    ProcessRequest {
        executable: executable.to_path_buf(),
        args: vec![
            OsString::from("--exact"),
            OsString::from("probe_fixture_entrypoint"),
            OsString::from("--nocapture"),
        ],
        timeout,
        environment: vec![(
            OsString::from("CODE_READY_PROCESS_FIXTURE"),
            OsString::from(fixture),
        )],
    }
}

#[test]
fn native_runner_caps_output_and_reports_timeout_and_nonzero() {
    let executable = std::env::current_exe().unwrap();
    let runner = NativeProcessRunner;

    let success = runner.run(fixture_request(
        &executable,
        "success",
        Duration::from_secs(3),
    ));
    assert!(matches!(
        success,
        ProcessOutcome::Exited {
            code: Some(0),
            stdout_truncated: true,
            ..
        }
    ));

    assert!(matches!(
        runner.run(fixture_request(
            &executable,
            "nonzero",
            Duration::from_secs(3)
        )),
        ProcessOutcome::Exited { code: Some(23), .. }
    ));
    assert_eq!(
        runner.run(fixture_request(
            &executable,
            "timeout",
            Duration::from_millis(30)
        )),
        ProcessOutcome::TimedOut
    );
}
