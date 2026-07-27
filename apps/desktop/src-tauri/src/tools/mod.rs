pub mod claude;
pub mod codex;
pub mod git;
pub mod shared;

pub use claude::boxed_claude_detector;
pub use codex::boxed_codex_detector;
pub use git::boxed_git_detector;
pub use shared::{Clock, DetectionContext, ToolDetector};

pub fn built_in_detector_registry() -> Vec<Box<dyn ToolDetector>> {
    vec![
        boxed_git_detector(),
        boxed_claude_detector(),
        boxed_codex_detector(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::contracts::{
        ObservedToolState, PlatformId, ToolCapability, ToolId, ToolObservation, ToolRequirement,
    };
    use crate::domain::detection::DETECTABLE_TOOL_IDS;
    use crate::domain::tool_registry::built_in_tool_registry;
    use crate::platform::fake::FakePlatformAdapter;
    use crate::platform::fake_process::FakeProcessRunner;
    use crate::platform::process::{ProcessOutcome, ProcessRequest};
    use crate::platform::{CandidateOrigin, ExecutableCandidate};
    use std::ffi::OsString;

    struct FixedClock;

    impl Clock for FixedClock {
        fn now_epoch_ms(&self) -> u64 {
            1_754_000_000_000
        }
    }

    fn native_path(path: &str) -> ExecutableCandidate {
        ExecutableCandidate::native(path, path, CandidateOrigin::Path)
    }

    fn success_for(tool_id: ToolId) -> ProcessOutcome {
        let output = match tool_id {
            ToolId::Git => "git version 2.47.1\n",
            ToolId::ClaudeCode => "2.1.89 (Claude Code)\n",
            ToolId::CodexCli => "codex-cli 0.138.0\n",
            ToolId::Winget | ToolId::Nodejs => "",
        };
        ProcessOutcome::Exited {
            code: Some(0),
            stdout: output.as_bytes().to_vec(),
            stderr: vec![],
            stdout_truncated: false,
            stderr_truncated: false,
        }
    }

    fn detect_one(
        detector: Box<dyn ToolDetector>,
        executable: &str,
    ) -> (ToolObservation, Vec<ProcessRequest>) {
        let tool_id = detector.tool_id();
        let adapter = FakePlatformAdapter::builder(PlatformId::MacosArm64)
            .with_candidates(tool_id.clone(), vec![native_path(executable)])
            .build();
        let runner = FakeProcessRunner::new([success_for(tool_id)]);
        let clock = FixedClock;
        let context = DetectionContext {
            platform: &adapter,
            runner: &runner,
            clock: &clock,
        };
        let observation = detector.detect(&context);
        (observation, runner.requests())
    }

    #[test]
    fn each_slice_one_detector_runs_only_its_native_version_command() {
        for (detector, executable) in [
            (boxed_git_detector(), "/fake/git"),
            (boxed_claude_detector(), "/fake/claude"),
            (boxed_codex_detector(), "/fake/codex"),
        ] {
            let (observation, requests) = detect_one(detector, executable);
            assert_eq!(requests.len(), 1);
            assert_eq!(requests[0].executable, std::path::PathBuf::from(executable));
            assert_eq!(requests[0].args, vec![OsString::from("--version")]);
            assert_eq!(observation.state, ObservedToolState::PresentHealthy);
        }
    }

    #[test]
    fn detector_registry_exactly_matches_slice_one_scope() {
        let ids = built_in_detector_registry()
            .iter()
            .map(|detector| detector.tool_id())
            .collect::<Vec<_>>();
        assert_eq!(ids, DETECTABLE_TOOL_IDS);
        assert_eq!(ids.len(), 3);
        assert!(!ids.contains(&ToolId::Winget));
        assert!(!ids.contains(&ToolId::Nodejs));
    }

    #[test]
    fn detector_registry_is_covered_by_the_product_registry_on_both_platforms() {
        let product_tools = built_in_tool_registry();

        for tool_id in built_in_detector_registry()
            .iter()
            .map(|detector| detector.tool_id())
        {
            let definition = product_tools
                .iter()
                .find(|tool| tool.id == tool_id)
                .expect("detector is a registered product tool");
            assert!(definition.capabilities.contains(&ToolCapability::Detect));
            assert!(definition.platform_policies.iter().all(|policy| {
                matches!(
                    policy.requirement,
                    ToolRequirement::Default | ToolRequirement::Optional
                )
            }));
        }

        assert_eq!(
            built_in_detector_registry().len(),
            DETECTABLE_TOOL_IDS.len()
        );
    }

    #[test]
    fn native_ai_detectors_do_not_reference_node_npm_or_shell_launchers() {
        let source_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/tools");
        let forbidden = [
            ["no", "de"].concat(),
            ["n", "pm"].concat(),
            [".", "cmd"].concat(),
            [".", "ps1"].concat(),
            ["power", "shell"].concat(),
            ["cmd", ".", "exe"].concat(),
            ["/bin", "/sh"].concat(),
        ];

        for module in ["claude.rs", "codex.rs"] {
            let source = std::fs::read_to_string(source_root.join(module))
                .expect("native AI detector source exists");
            for token in &forbidden {
                assert!(
                    !source.to_ascii_lowercase().contains(token),
                    "{module} contains {token}"
                );
            }
        }
    }
}
