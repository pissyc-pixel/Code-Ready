#![allow(dead_code)]

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::{
    config::InstallNetworkConfig,
    detector::{
        current_npm_status, has_node_runtime, npm_global_probe_path, recheck_ai_npm_command,
        ToolInstallStatus, ToolStatus,
    },
    network::npm_registry::npm_registry_args,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AiNpmPackageId {
    Claude,
    Codex,
    Opencode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NpmPackageSpec {
    pub id: AiNpmPackageId,
    pub package_name: &'static str,
    pub command_name: &'static str,
    pub display_name: &'static str,
    pub detect_after: &'static [&'static str],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NpmDependencyIssue {
    NodeMissing,
    NpmMissing,
    NpmPathMissing(PathBuf),
}

impl NpmDependencyIssue {
    pub fn error_message(&self) -> String {
        match self {
            Self::NodeMissing => "Node.js 未安装，无法启动 npm 安装任务。".to_string(),
            Self::NpmMissing => "npm 未安装，无法启动 npm 安装任务。".to_string(),
            Self::NpmPathMissing(path) => format!(
                "已探测到 npm 全局路径 {}，但当前 PATH 中无法直接调用 npm。",
                path.display()
            ),
        }
    }

    pub fn suggestion(&self) -> String {
        match self {
            Self::NodeMissing => "请先安装 Node.js LTS，再安装 AI Coding 工具。".to_string(),
            Self::NpmMissing => "请先安装包含 npm 的 Node.js，再重试。".to_string(),
            Self::NpmPathMissing(_) => {
                "当前仅提示 PATH 可能未刷新；V0.5 会补 PATH 修复说明，不会自动修改系统 PATH。"
                    .to_string()
            }
        }
    }
}

pub fn package_spec(id: AiNpmPackageId) -> NpmPackageSpec {
    match id {
        AiNpmPackageId::Claude => NpmPackageSpec {
            id,
            package_name: "@anthropic-ai/claude-code",
            command_name: "claude",
            display_name: "Claude Code",
            detect_after: &["claude"],
        },
        AiNpmPackageId::Codex => NpmPackageSpec {
            id,
            package_name: "@openai/codex",
            command_name: "codex",
            display_name: "Codex CLI",
            detect_after: &["codex"],
        },
        AiNpmPackageId::Opencode => NpmPackageSpec {
            id,
            package_name: "opencode-ai",
            command_name: "opencode",
            display_name: "OpenCode",
            detect_after: &["opencode"],
        },
    }
}

pub fn build_global_install_args(
    package: &NpmPackageSpec,
    config: &InstallNetworkConfig,
) -> Vec<String> {
    build_global_install_args_with_target(package.package_name, config)
}

pub fn build_global_install_args_with_target(
    package_target: &str,
    config: &InstallNetworkConfig,
) -> Vec<String> {
    let mut args = vec![
        "install".to_string(),
        "-g".to_string(),
        package_target.to_string(),
    ];
    args.extend(npm_registry_args(config));
    args
}

pub fn check_npm_install_prerequisites() -> Result<(), NpmDependencyIssue> {
    if !has_node_runtime().map_err(|_| NpmDependencyIssue::NodeMissing)? {
        return Err(NpmDependencyIssue::NodeMissing);
    }

    let npm_status = current_npm_status();

    if matches!(npm_status.status, ToolInstallStatus::Installed) {
        return Ok(());
    }

    if matches!(npm_status.status, ToolInstallStatus::InstalledButPathMissing) {
        let path = npm_global_probe_path("npm")
            .map_err(|_| NpmDependencyIssue::NpmMissing)?
            .ok_or(NpmDependencyIssue::NpmMissing)?;
        return Err(NpmDependencyIssue::NpmPathMissing(path));
    }

    match npm_global_probe_path("npm").map_err(|_| NpmDependencyIssue::NpmMissing)? {
        Some(path) => Err(NpmDependencyIssue::NpmPathMissing(path)),
        None => Err(NpmDependencyIssue::NpmMissing),
    }
}

pub fn recheck_npm_global_command(command_name: &str, display_name: &str) -> ToolStatus {
    recheck_ai_npm_command(command_name, display_name)
}

#[cfg(test)]
mod tests {
    use super::{
        build_global_install_args, build_global_install_args_with_target, package_spec,
        AiNpmPackageId, NpmDependencyIssue,
    };
    use crate::config::{InstallNetworkConfig, InstallNetworkMode, NpmRegistryOption};

    #[test]
    fn builds_default_registry_install_args() {
        let args = build_global_install_args(
            &package_spec(AiNpmPackageId::Claude),
            &InstallNetworkConfig {
                mode: InstallNetworkMode::None,
                proxy_url: None,
                npm_registry: NpmRegistryOption::Default,
                custom_npm_registry: None,
            },
        );

        assert_eq!(
            args,
            vec![
                "install".to_string(),
                "-g".to_string(),
                "@anthropic-ai/claude-code".to_string(),
            ]
        );
    }

    #[test]
    fn builds_npmmirror_install_args() {
        let args = build_global_install_args(
            &package_spec(AiNpmPackageId::Codex),
            &InstallNetworkConfig {
                mode: InstallNetworkMode::ManualProxy,
                proxy_url: Some("http://127.0.0.1:7890".to_string()),
                npm_registry: NpmRegistryOption::Npmmirror,
                custom_npm_registry: None,
            },
        );

        assert_eq!(
            args,
            vec![
                "install".to_string(),
                "-g".to_string(),
                "@openai/codex".to_string(),
                "--registry=https://registry.npmmirror.com".to_string(),
            ]
        );
    }

    #[test]
    fn builds_custom_registry_install_args() {
        let args = build_global_install_args(
            &package_spec(AiNpmPackageId::Opencode),
            &InstallNetworkConfig {
                mode: InstallNetworkMode::SystemProxy,
                proxy_url: None,
                npm_registry: NpmRegistryOption::Custom,
                custom_npm_registry: Some("https://registry.example.com".to_string()),
            },
        );

        assert_eq!(
            args,
            vec![
                "install".to_string(),
                "-g".to_string(),
                "opencode-ai".to_string(),
                "--registry=https://registry.example.com".to_string(),
            ]
        );
    }

    #[test]
    fn builds_latest_target_install_args() {
        let args = build_global_install_args_with_target(
            "@openai/codex@latest",
            &InstallNetworkConfig {
                mode: InstallNetworkMode::ManualProxy,
                proxy_url: Some("http://127.0.0.1:7890".to_string()),
                npm_registry: NpmRegistryOption::Default,
                custom_npm_registry: None,
            },
        );

        assert_eq!(
            args,
            vec![
                "install".to_string(),
                "-g".to_string(),
                "@openai/codex@latest".to_string(),
            ]
        );
    }

    #[test]
    fn package_specs_cover_ai_tools() {
        let claude = package_spec(AiNpmPackageId::Claude);
        let codex = package_spec(AiNpmPackageId::Codex);
        let opencode = package_spec(AiNpmPackageId::Opencode);

        assert_eq!(claude.command_name, "claude");
        assert_eq!(codex.command_name, "codex");
        assert_eq!(opencode.command_name, "opencode");
    }

    #[test]
    fn dependency_issue_messages_are_actionable() {
        assert!(NpmDependencyIssue::NodeMissing
            .suggestion()
            .contains("Node.js"));
        assert!(NpmDependencyIssue::NpmMissing.error_message().contains("npm"));
    }
}
