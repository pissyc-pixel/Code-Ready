use crate::config::{InstallNetworkConfig, NpmRegistryOption};

#[allow(dead_code)]
pub fn npm_registry_args(config: &InstallNetworkConfig) -> Vec<String> {
    match config.npm_registry {
        NpmRegistryOption::Default => Vec::new(),
        NpmRegistryOption::Npmmirror => {
            vec!["--registry=https://registry.npmmirror.com".to_string()]
        }
        NpmRegistryOption::Custom => config
            .custom_npm_registry
            .as_ref()
            .map(|registry| vec![format!("--registry={registry}")])
            .unwrap_or_default(),
    }
}

#[allow(dead_code)]
pub fn append_npm_registry_args(
    args: &mut Vec<String>,
    config: &InstallNetworkConfig,
) {
    args.extend(npm_registry_args(config));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{InstallNetworkMode, NpmRegistryOption};

    #[test]
    fn adds_npmmirror_registry_as_single_use_flag() {
        let config = InstallNetworkConfig {
            mode: InstallNetworkMode::None,
            proxy_url: None,
            npm_registry: NpmRegistryOption::Npmmirror,
            custom_npm_registry: None,
        };

        assert_eq!(
            npm_registry_args(&config),
            vec!["--registry=https://registry.npmmirror.com".to_string()]
        );
    }

    #[test]
    fn leaves_default_registry_empty() {
        let config = InstallNetworkConfig {
            mode: InstallNetworkMode::None,
            proxy_url: None,
            npm_registry: NpmRegistryOption::Default,
            custom_npm_registry: None,
        };

        assert!(npm_registry_args(&config).is_empty());
    }

    #[test]
    fn appends_registry_args_without_writing_global_config() {
        let config = InstallNetworkConfig {
            mode: InstallNetworkMode::ManualProxy,
            proxy_url: Some("http://127.0.0.1:7890".to_string()),
            npm_registry: NpmRegistryOption::Custom,
            custom_npm_registry: Some("https://registry.example.com".to_string()),
        };
        let mut args = vec!["install".to_string(), "-g".to_string(), "pkg".to_string()];

        append_npm_registry_args(&mut args, &config);

        assert_eq!(
            args,
            vec![
                "install".to_string(),
                "-g".to_string(),
                "pkg".to_string(),
                "--registry=https://registry.example.com".to_string()
            ]
        );
        assert!(!args.iter().any(|item| item.contains("npm config set registry")));
    }
}
