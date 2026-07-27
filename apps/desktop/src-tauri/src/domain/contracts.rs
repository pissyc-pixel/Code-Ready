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
#[ts(rename_all = "camelCase", export_to = "BootstrapState.ts")]
pub struct BootstrapState {
    pub schema_version: u16,
    pub platform: PlatformId,
    pub tools: Vec<ToolDefinition>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bootstrap_contract_serializes_for_frontend() {
        let state = BootstrapState {
            schema_version: 1,
            platform: PlatformId::MacosArm64,
            tools: vec![],
        };

        assert_eq!(
            serde_json::to_value(state).unwrap(),
            serde_json::json!({
                "schemaVersion": 1,
                "platform": "macosArm64",
                "tools": []
            })
        );
    }
}
