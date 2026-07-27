use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase", export_to = "PlatformId.ts")]
pub enum PlatformId {
    WindowsX64,
    MacosArm64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase", export_to = "ToolId.ts")]
pub enum ToolId {
    Winget,
    Git,
    Nodejs,
    ClaudeCode,
    CodexCli,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase", export_to = "ToolRequirement.ts")]
pub enum ToolRequirement {
    Default,
    Optional,
    Unavailable,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase", export_to = "ToolCapability.ts")]
pub enum ToolCapability {
    Detect,
    Install,
    Upgrade,
    Repair,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase", export_to = "PlatformPolicy.ts")]
pub struct PlatformPolicy {
    pub platform: PlatformId,
    pub requirement: ToolRequirement,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase", export_to = "ToolDefinition.ts")]
pub struct ToolDefinition {
    pub id: ToolId,
    pub label_key: String,
    pub platform_policies: Vec<PlatformPolicy>,
    pub capabilities: Vec<ToolCapability>,
    pub runtime_dependencies: Vec<ToolId>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase", export_to = "ObservedToolState.ts")]
pub enum ObservedToolState {
    Absent,
    PresentHealthy,
    PresentPathIssue,
    PresentBroken,
    Unknown,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase", export_to = "VersionStatus.ts")]
pub enum VersionStatus {
    Current,
    Outdated,
    NewerThanKnown,
    NotComparable,
    Unknown,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase", export_to = "DetectionEvidenceCode.ts")]
pub enum DetectionEvidenceCode {
    PathCommandHealthy,
    KnownLocationHealthy,
    PathShadowed,
    NonNativeLauncher,
    CommandNonZero,
    CommandTimedOut,
    CommandLaunchFailed,
    VersionUnparseable,
    NotFound,
    PlatformProbeFailed,
    AppleDeveloperToolsMissing,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase", export_to = "ProcessExitKind.ts")]
pub enum ProcessExitKind {
    Success,
    NonZero,
    Signalled,
    TimedOut,
    LaunchFailed,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase", export_to = "DetectionEvidence.ts")]
pub struct DetectionEvidence {
    pub code: DetectionEvidenceCode,
    pub display_path: Option<String>,
    pub exit: Option<ProcessExitKind>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase", export_to = "ToolObservation.ts")]
pub struct ToolObservation {
    pub tool_id: ToolId,
    pub state: ObservedToolState,
    pub version: Option<String>,
    pub version_status: VersionStatus,
    pub evidence: DetectionEvidence,
    #[ts(type = "number")]
    pub checked_at_epoch_ms: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase", export_to = "DetectionRunStatus.ts")]
pub enum DetectionRunStatus {
    Running,
    Completed,
    Failed,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase", export_to = "DetectionRunErrorCode.ts")]
pub enum DetectionRunErrorCode {
    Internal,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase", export_to = "DetectionRun.ts")]
pub struct DetectionRun {
    pub id: String,
    pub requested_tool_ids: Vec<ToolId>,
    pub status: DetectionRunStatus,
    #[ts(type = "number")]
    pub started_at_epoch_ms: u64,
    #[ts(type = "number | null")]
    pub finished_at_epoch_ms: Option<u64>,
    pub error_code: Option<DetectionRunErrorCode>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase", export_to = "AppSnapshot.ts")]
pub struct AppSnapshot {
    pub schema_version: u16,
    #[ts(type = "number")]
    pub snapshot_version: u64,
    #[ts(type = "number")]
    pub last_event_sequence: u64,
    pub platform: PlatformId,
    pub tools: Vec<ToolDefinition>,
    pub observations: Vec<ToolObservation>,
    pub detection_run: Option<DetectionRun>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase", export_to = "DetectionEventType.ts")]
pub enum DetectionEventType {
    #[serde(rename = "detection.changed")]
    #[ts(rename = "detection.changed")]
    DetectionChanged,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase", export_to = "DetectionEventEnvelope.ts")]
pub struct DetectionEventEnvelope {
    pub schema_version: u16,
    #[ts(type = "number")]
    pub sequence: u64,
    #[ts(type = "number")]
    pub snapshot_version: u64,
    #[ts(type = "number")]
    pub emitted_at_epoch_ms: u64,
    pub event_type: DetectionEventType,
    pub run_id: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase", export_to = "CommandErrorCode.ts")]
pub enum CommandErrorCode {
    InvalidToolSelection,
    DetectionAlreadyRunning,
    Internal,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase", export_to = "CommandError.ts")]
pub struct CommandError {
    pub code: CommandErrorCode,
    pub retryable: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slice_one_snapshot_keeps_fact_version_and_run_state_separate() {
        let snapshot = AppSnapshot {
            schema_version: 2,
            snapshot_version: 7,
            last_event_sequence: 11,
            platform: PlatformId::MacosArm64,
            tools: vec![],
            observations: vec![ToolObservation {
                tool_id: ToolId::ClaudeCode,
                state: ObservedToolState::PresentPathIssue,
                version: Some("2.1.89".into()),
                version_status: VersionStatus::NotComparable,
                evidence: DetectionEvidence {
                    code: DetectionEvidenceCode::KnownLocationHealthy,
                    display_path: Some("~/.local/bin/claude".into()),
                    exit: Some(ProcessExitKind::Success),
                },
                checked_at_epoch_ms: 1_754_000_000_000,
            }],
            detection_run: Some(DetectionRun {
                id: "run-1".into(),
                requested_tool_ids: vec![ToolId::ClaudeCode],
                status: DetectionRunStatus::Running,
                started_at_epoch_ms: 1_754_000_000_000,
                finished_at_epoch_ms: None,
                error_code: None,
            }),
        };

        assert_eq!(
            serde_json::to_value(snapshot).unwrap(),
            serde_json::json!({
                "schemaVersion": 2,
                "snapshotVersion": 7,
                "lastEventSequence": 11,
                "platform": "macosArm64",
                "tools": [],
                "observations": [{
                    "toolId": "claudeCode",
                    "state": "presentPathIssue",
                    "version": "2.1.89",
                    "versionStatus": "notComparable",
                    "evidence": {
                        "code": "knownLocationHealthy",
                        "displayPath": "~/.local/bin/claude",
                        "exit": "success"
                    },
                    "checkedAtEpochMs": 1_754_000_000_000_u64
                }],
                "detectionRun": {
                    "id": "run-1",
                    "requestedToolIds": ["claudeCode"],
                    "status": "running",
                    "startedAtEpochMs": 1_754_000_000_000_u64,
                    "finishedAtEpochMs": null,
                    "errorCode": null
                }
            })
        );
    }

    #[test]
    fn detection_event_envelope_uses_the_stable_event_name() {
        let event = DetectionEventEnvelope {
            schema_version: 1,
            sequence: 12,
            snapshot_version: 8,
            emitted_at_epoch_ms: 1_754_000_000_100,
            event_type: DetectionEventType::DetectionChanged,
            run_id: "run-1".into(),
        };

        assert_eq!(
            serde_json::to_value(event).unwrap()["eventType"],
            "detection.changed"
        );
    }
}
