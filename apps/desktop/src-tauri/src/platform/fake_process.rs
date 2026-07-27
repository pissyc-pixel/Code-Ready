use std::collections::VecDeque;
use std::sync::Mutex;

use super::process::{LaunchFailureKind, ProcessOutcome, ProcessRequest, ProcessRunner};

pub struct FakeProcessRunner {
    outcomes: Mutex<VecDeque<ProcessOutcome>>,
    requests: Mutex<Vec<ProcessRequest>>,
}

impl FakeProcessRunner {
    pub fn new(outcomes: impl IntoIterator<Item = ProcessOutcome>) -> Self {
        Self {
            outcomes: Mutex::new(outcomes.into_iter().collect()),
            requests: Mutex::new(vec![]),
        }
    }

    pub fn requests(&self) -> Vec<ProcessRequest> {
        self.requests
            .lock()
            .expect("fake request mutex is not poisoned")
            .clone()
    }
}

impl ProcessRunner for FakeProcessRunner {
    fn run(&self, request: ProcessRequest) -> ProcessOutcome {
        self.requests
            .lock()
            .expect("fake request mutex is not poisoned")
            .push(request);
        self.outcomes
            .lock()
            .expect("fake outcome mutex is not poisoned")
            .pop_front()
            .unwrap_or(ProcessOutcome::LaunchFailed(LaunchFailureKind::Other))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::time::Duration;

    #[test]
    fn fake_process_runner_returns_queued_results_and_records_exact_request() {
        let runner = FakeProcessRunner::new([ProcessOutcome::Exited {
            code: Some(0),
            stdout: b"codex-cli 0.138.0\n".to_vec(),
            stderr: vec![],
            stdout_truncated: false,
            stderr_truncated: false,
        }]);
        let request = ProcessRequest::version_probe("/fake/codex");

        assert!(matches!(
            runner.run(request.clone()),
            ProcessOutcome::Exited { code: Some(0), .. }
        ));
        assert_eq!(runner.requests(), vec![request]);
    }

    #[test]
    fn version_probe_has_no_shell_or_mutating_arguments() {
        let request = ProcessRequest::version_probe("/fake/claude");
        assert_eq!(request.executable, PathBuf::from("/fake/claude"));
        assert_eq!(request.args, vec!["--version"]);
        assert_eq!(request.timeout, Duration::from_secs(3));
        assert!(request.environment.is_empty());
    }
}
