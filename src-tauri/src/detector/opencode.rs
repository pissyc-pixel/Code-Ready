use super::{auth_cli, ToolStatus};

pub fn detect() -> ToolStatus {
    auth_cli::detect_auth_sensitive_cli("opencode", "OpenCode")
}
