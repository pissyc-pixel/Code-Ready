use crate::config::{AppConfig, InstallNetworkMode};

use super::runner::InstallCommandSpec;

pub fn command_spec(config: &AppConfig) -> InstallCommandSpec {
    InstallCommandSpec {
        program: "winget".to_string(),
        args: vec![
            "install".to_string(),
            "--id".to_string(),
            "OpenJS.NodeJS.LTS".to_string(),
            "-e".to_string(),
            "--source".to_string(),
            "winget".to_string(),
            "--accept-package-agreements".to_string(),
            "--accept-source-agreements".to_string(),
        ],
        envs: Vec::new(),
        timeout: std::time::Duration::from_secs(30 * 60),
        started_suggestion: winget_proxy_warning(config),
        success_suggestion: Some("Node.js 安装完成，正在重新检测 Node.js 与 npm。".to_string()),
        failure_suggestion: Some(
            "如果日志提示权限不足或 UAC 被拒绝，请以管理员身份重启后重试。".to_string(),
        ),
        detect_after: vec!["node".to_string(), "npm".to_string()],
    }
}

fn winget_proxy_warning(config: &AppConfig) -> Option<String> {
    if matches!(config.install_network.mode, InstallNetworkMode::ManualProxy) {
        return Some(
            "winget 不保证读取本客户端的临时代理设置。如下载失败，请先在节点软件中开启系统代理，或后续使用安装包兜底方案。"
                .to_string(),
        );
    }

    None
}

#[cfg(test)]
mod tests {
    use super::command_spec;
    use crate::config::{AppConfig, InstallNetworkConfig, InstallNetworkMode, NpmRegistryOption};

    #[test]
    fn builds_node_winget_command() {
        let spec = command_spec(&AppConfig::default());
        assert_eq!(spec.program, "winget");
        assert_eq!(
            spec.args,
            vec![
                "install",
                "--id",
                "OpenJS.NodeJS.LTS",
                "-e",
                "--source",
                "winget",
                "--accept-package-agreements",
                "--accept-source-agreements"
            ]
        );
    }

    #[test]
    fn adds_manual_proxy_warning_for_node() {
        let spec = command_spec(&AppConfig {
            install_network: InstallNetworkConfig {
                mode: InstallNetworkMode::ManualProxy,
                proxy_url: Some("http://127.0.0.1:7890".to_string()),
                npm_registry: NpmRegistryOption::Default,
                custom_npm_registry: None,
            },
        });

        assert!(spec
            .started_suggestion
            .expect("manual proxy warning")
            .contains("winget 不保证读取本客户端的临时代理设置"));
    }
}
