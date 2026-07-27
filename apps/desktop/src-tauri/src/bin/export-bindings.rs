use std::fs;
use std::path::{Path, PathBuf};

use code_ready_desktop_lib::domain::contracts::{
    AppSnapshot, CommandError, CommandErrorCode, DetectionEventEnvelope, DetectionEventType,
    DetectionEvidence, DetectionEvidenceCode, DetectionRun, DetectionRunErrorCode,
    DetectionRunStatus, ObservedToolState, PlatformId, PlatformPolicy, ProcessExitKind,
    ToolCapability, ToolDefinition, ToolId, ToolObservation, ToolRequirement, VersionStatus,
};
use ts_rs::{Config, TS};

const CONTRACT_TYPE_NAMES: [&str; 20] = [
    "AppSnapshot",
    "CommandError",
    "CommandErrorCode",
    "DetectionEventEnvelope",
    "DetectionEventType",
    "DetectionEvidence",
    "DetectionEvidenceCode",
    "DetectionRun",
    "DetectionRunErrorCode",
    "DetectionRunStatus",
    "ObservedToolState",
    "PlatformId",
    "PlatformPolicy",
    "ProcessExitKind",
    "ToolCapability",
    "ToolDefinition",
    "ToolId",
    "ToolObservation",
    "ToolRequirement",
    "VersionStatus",
];

#[cfg(test)]
fn exported_type_names() -> Vec<&'static str> {
    CONTRACT_TYPE_NAMES.to_vec()
}

fn generated_directory() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../src/shared/api/generated")
}

fn clean_generated_types(directory: &Path) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(directory)?;

    for entry in fs::read_dir(directory)? {
        let path = entry?.path();
        if path.extension().and_then(|extension| extension.to_str()) == Some("ts") {
            fs::remove_file(path)?;
        }
    }

    Ok(())
}

fn export_bindings() -> Result<(), Box<dyn std::error::Error>> {
    let directory = generated_directory();
    clean_generated_types(&directory)?;
    let config = Config::new().with_out_dir(&directory);

    AppSnapshot::export(&config)?;
    CommandError::export(&config)?;
    CommandErrorCode::export(&config)?;
    DetectionEventEnvelope::export(&config)?;
    DetectionEventType::export(&config)?;
    DetectionEvidence::export(&config)?;
    DetectionEvidenceCode::export(&config)?;
    DetectionRun::export(&config)?;
    DetectionRunErrorCode::export(&config)?;
    DetectionRunStatus::export(&config)?;
    ObservedToolState::export(&config)?;
    PlatformId::export(&config)?;
    PlatformPolicy::export(&config)?;
    ProcessExitKind::export(&config)?;
    ToolCapability::export(&config)?;
    ToolDefinition::export(&config)?;
    ToolId::export(&config)?;
    ToolObservation::export(&config)?;
    ToolRequirement::export(&config)?;
    VersionStatus::export(&config)?;

    let index = CONTRACT_TYPE_NAMES
        .iter()
        .map(|name| format!("export type {{ {name} }} from \"./{name}\";\n"))
        .collect::<String>();
    fs::write(directory.join("index.ts"), index)?;

    Ok(())
}

fn main() {
    export_bindings().expect("export Rust IPC bindings");
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use super::exported_type_names;

    #[test]
    fn export_bindings_have_exact_contract_type_set_and_no_legacy_generated_files() {
        assert_eq!(
            exported_type_names(),
            vec![
                "AppSnapshot",
                "CommandError",
                "CommandErrorCode",
                "DetectionEventEnvelope",
                "DetectionEventType",
                "DetectionEvidence",
                "DetectionEvidenceCode",
                "DetectionRun",
                "DetectionRunErrorCode",
                "DetectionRunStatus",
                "ObservedToolState",
                "PlatformId",
                "PlatformPolicy",
                "ProcessExitKind",
                "ToolCapability",
                "ToolDefinition",
                "ToolId",
                "ToolObservation",
                "ToolRequirement",
                "VersionStatus",
            ]
        );

        let generated_directory =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../src/shared/api/generated");
        let generated_files = fs::read_dir(generated_directory)
            .map(|entries| {
                entries
                    .filter_map(Result::ok)
                    .map(|entry| entry.file_name().to_string_lossy().into_owned())
                    .filter(|name| name.ends_with(".ts"))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        assert!(generated_files.iter().all(|file| {
            [
                "AppSnapshot.ts",
                "CommandError.ts",
                "CommandErrorCode.ts",
                "DetectionEventEnvelope.ts",
                "DetectionEventType.ts",
                "DetectionEvidence.ts",
                "DetectionEvidenceCode.ts",
                "DetectionRun.ts",
                "DetectionRunErrorCode.ts",
                "DetectionRunStatus.ts",
                "ObservedToolState.ts",
                "PlatformId.ts",
                "PlatformPolicy.ts",
                "ProcessExitKind.ts",
                "ToolCapability.ts",
                "ToolDefinition.ts",
                "ToolId.ts",
                "ToolObservation.ts",
                "ToolRequirement.ts",
                "VersionStatus.ts",
                "index.ts",
            ]
            .contains(&file.as_str())
        }));
    }
}
