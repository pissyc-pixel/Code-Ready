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
}
