use std::time::Duration;

use crate::{config::AppConfig, network::proxy_env::proxy_env_vars};

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
    if check_prerequisites {
        check_npm_install_prerequisites()
            .map_err(|issue| format!("{} {}", issue.error_message(), issue.suggestion()))?;
    }

    let package = package_spec(AiNpmPackageId::Opencode);
    let package_target = if matches!(mode, InstallRequestMode::Latest) {
        format!("{}@latest", package.package_name)
    } else {
        package.package_name.to_string()
    };
    Ok(InstallCommandSpec {
        program: "npm".to_string(),
        args: if package_target == package.package_name {
            build_global_install_args(&package, &config.install_network)
        } else {
            build_global_install_args_with_target(&package_target, &config.install_network)
        },
        envs: proxy_env_vars(&config.install_network),
        timeout: Duration::from_secs(20 * 60),
        started_suggestion: Some("OpenCode will be installed via global npm.".to_string()),
        success_suggestion: Some(
            "OpenCode installation finished; re-checking command availability.".to_string(),
        ),
        failure_suggestion: Some(
            "If OpenCode installation fails, check Node.js, npm, or the selected registry."
                .to_string(),
        ),
        detect_after: package
            .detect_after
            .iter()
            .map(|item| item.to_string())
            .collect(),
    })
}

#[cfg(test)]
mod tests {
    use super::command_spec_with_prerequisite_check;
    use crate::config::{AppConfig, InstallNetworkConfig, InstallNetworkMode, NpmRegistryOption};
    use crate::installer::InstallRequestMode;

    #[test]
    fn builds_opencode_npm_install_command() {
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
        },
            false,
            InstallRequestMode::Install,
        )
        .expect("opencode install spec");

        assert_eq!(spec.program, "npm");
        assert_eq!(
            spec.args,
            vec![
                "install".to_string(),
                "-g".to_string(),
                "opencode-ai".to_string(),
                "--registry=https://registry.npmmirror.com".to_string(),
            ]
        );
        assert_eq!(spec.detect_after, vec!["opencode".to_string()]);
    }

    #[test]
    fn builds_opencode_latest_install_command() {
        let spec = command_spec_with_prerequisite_check(
            &AppConfig::default(),
            false,
            InstallRequestMode::Latest,
        )
        .expect("opencode latest spec");

        assert_eq!(
            spec.args,
            vec![
                "install".to_string(),
                "-g".to_string(),
                "opencode-ai@latest".to_string(),
            ]
        );
    }
}
