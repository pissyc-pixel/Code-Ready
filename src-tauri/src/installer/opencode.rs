use std::time::Duration;

use crate::{config::AppConfig, network::proxy_env::proxy_env_vars};

use super::{
    npm::{
        build_global_install_args, check_npm_install_prerequisites, package_spec, AiNpmPackageId,
    },
    runner::InstallCommandSpec,
};

pub fn command_spec(config: &AppConfig) -> Result<InstallCommandSpec, String> {
    command_spec_with_prerequisite_check(config, true)
}

fn command_spec_with_prerequisite_check(
    config: &AppConfig,
    check_prerequisites: bool,
) -> Result<InstallCommandSpec, String> {
    if check_prerequisites {
        check_npm_install_prerequisites()
            .map_err(|issue| format!("{} {}", issue.error_message(), issue.suggestion()))?;
    }

    let package = package_spec(AiNpmPackageId::Opencode);
    Ok(InstallCommandSpec {
        program: "npm".to_string(),
        args: build_global_install_args(&package, &config.install_network),
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
}
