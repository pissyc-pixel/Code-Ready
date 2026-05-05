use std::time::Duration;

use crate::{
    config::{AppConfig, InstallNetworkMode},
    network::proxy_env::proxy_env_vars,
};

use super::{
    npm::{
        build_global_install_args, build_global_install_args_with_target,
        check_npm_install_prerequisites, package_spec, AiNpmPackageId,
    },
    runner::InstallCommandSpec,
    InstallRequestMode,
};

#[allow(dead_code)]
pub fn command_spec(config: &AppConfig) -> Result<InstallCommandSpec, String> {
    command_spec_for_mode(config, InstallRequestMode::Install)
}

pub fn command_spec_for_mode(
    config: &AppConfig,
    mode: InstallRequestMode,
) -> Result<InstallCommandSpec, String> {
    command_spec_with_prerequisite_check(config, true, mode)
}

fn command_spec_with_prerequisite_check(
    config: &AppConfig,
    check_prerequisites: bool,
    mode: InstallRequestMode,
) -> Result<InstallCommandSpec, String> {
    if matches!(config.install_network.mode, InstallNetworkMode::ManualProxy)
        || matches!(mode, InstallRequestMode::Latest)
    {
        if check_prerequisites {
            check_npm_install_prerequisites()
                .map_err(|issue| format!("{} {}", issue.error_message(), issue.suggestion()))?;
        }

        let package = package_spec(AiNpmPackageId::Claude);
        let package_target = if matches!(mode, InstallRequestMode::Latest) {
            format!("{}@latest", package.package_name)
        } else {
            package.package_name.to_string()
        };
        return Ok(InstallCommandSpec {
            program: "npm".to_string(),
            args: if matches!(mode, InstallRequestMode::Install | InstallRequestMode::Reinstall)
                && package_target == package.package_name
            {
                build_global_install_args(&package, &config.install_network)
            } else {
                build_global_install_args_with_target(&package_target, &config.install_network)
            },
            envs: proxy_env_vars(&config.install_network),
            timeout: Duration::from_secs(20 * 60),
            started_suggestion: Some(
                if matches!(mode, InstallRequestMode::Latest) {
                    "Claude Code latest install uses the npm fallback with @latest."
                } else {
                    "manual_proxy mode prefers the npm fallback for Claude Code."
                }
                .to_string(),
            ),
            success_suggestion: Some(
                "Claude Code installation finished; re-checking command availability."
                    .to_string(),
            ),
            failure_suggestion: Some(
                "If npm install fails, check Node.js, npm, registry, or temporary proxy settings."
                    .to_string(),
            ),
            detect_after: package
                .detect_after
                .iter()
                .map(|item| item.to_string())
                .collect(),
        });
    }

    Ok(InstallCommandSpec {
        program: "powershell.exe".to_string(),
        args: vec![
            "-NoProfile".to_string(),
            "-ExecutionPolicy".to_string(),
            "Bypass".to_string(),
            "-Command".to_string(),
            "irm https://claude.ai/install.ps1 | iex".to_string(),
        ],
        envs: proxy_env_vars(&config.install_network),
        timeout: Duration::from_secs(20 * 60),
        started_suggestion: Some(
            "Claude Code will use the official PowerShell native installer.".to_string(),
        ),
        success_suggestion: Some(
            "Claude Code installation finished; re-checking command availability."
                .to_string(),
        ),
        failure_suggestion: Some(
            "If the official installer fails, check network access or retry with manual_proxy npm fallback."
                .to_string(),
        ),
        detect_after: vec!["claude".to_string()],
    })
}

#[cfg(test)]
mod tests {
    use super::{command_spec, command_spec_with_prerequisite_check};
    use crate::config::{AppConfig, InstallNetworkConfig, InstallNetworkMode, NpmRegistryOption};
    use crate::installer::InstallRequestMode;

    #[test]
    fn uses_npm_fallback_for_claude_in_manual_proxy_mode() {
        let spec = command_spec_with_prerequisite_check(
            &AppConfig {
            install_network: InstallNetworkConfig {
                mode: InstallNetworkMode::ManualProxy,
                proxy_url: Some("http://127.0.0.1:7890".to_string()),
                npm_registry: NpmRegistryOption::Npmmirror,
                custom_npm_registry: None,
            },
            ccswitch_path: None,
            ccswitch_download_sources: Vec::new(),
            subscription_page_url: None,
        },
            false,
            InstallRequestMode::Install,
        )
        .expect("manual proxy should build npm fallback");

        assert_eq!(spec.program, "npm");
        assert_eq!(
            spec.args,
            vec![
                "install".to_string(),
                "-g".to_string(),
                "@anthropic-ai/claude-code".to_string(),
                "--registry=https://registry.npmmirror.com".to_string(),
            ]
        );
        assert!(spec
            .started_suggestion
            .expect("started suggestion")
            .contains("npm fallback"));
        assert!(spec.envs.iter().any(|(key, _)| key == "HTTP_PROXY"));
    }

    #[test]
    fn uses_native_installer_with_execution_policy_bypass_by_default() {
        let spec = command_spec(&AppConfig::default()).expect("default native install spec");

        assert_eq!(spec.program, "powershell.exe");
        assert_eq!(
            spec.args,
            vec![
                "-NoProfile".to_string(),
                "-ExecutionPolicy".to_string(),
                "Bypass".to_string(),
                "-Command".to_string(),
                "irm https://claude.ai/install.ps1 | iex".to_string(),
            ]
        );
        assert_eq!(spec.detect_after, vec!["claude".to_string()]);
    }

    #[test]
    fn latest_claude_install_uses_npm_latest_target() {
        let spec = command_spec_with_prerequisite_check(
            &AppConfig::default(),
            false,
            InstallRequestMode::Latest,
        )
        .expect("latest mode should use npm");

        assert_eq!(spec.program, "npm");
        assert_eq!(
            spec.args,
            vec![
                "install".to_string(),
                "-g".to_string(),
                "@anthropic-ai/claude-code@latest".to_string(),
            ]
        );
    }
}
