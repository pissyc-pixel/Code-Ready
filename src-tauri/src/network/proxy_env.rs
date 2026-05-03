use crate::config::{InstallNetworkConfig, InstallNetworkMode};

pub fn proxy_env_vars(config: &InstallNetworkConfig) -> Vec<(String, String)> {
    match (&config.mode, config.proxy_url.as_ref()) {
        (InstallNetworkMode::ManualProxy, Some(proxy_url)) if !proxy_url.trim().is_empty() => vec![
            ("HTTP_PROXY".to_string(), proxy_url.clone()),
            ("HTTPS_PROXY".to_string(), proxy_url.clone()),
            ("ALL_PROXY".to_string(), proxy_url.clone()),
        ],
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::proxy_env_vars;
    use crate::config::{InstallNetworkConfig, InstallNetworkMode, NpmRegistryOption};

    #[test]
    fn returns_manual_proxy_envs_as_single_process_settings() {
        let envs = proxy_env_vars(&InstallNetworkConfig {
            mode: InstallNetworkMode::ManualProxy,
            proxy_url: Some("http://127.0.0.1:7890".to_string()),
            npm_registry: NpmRegistryOption::Default,
            custom_npm_registry: None,
        });

        assert_eq!(
            envs,
            vec![
                ("HTTP_PROXY".to_string(), "http://127.0.0.1:7890".to_string()),
                ("HTTPS_PROXY".to_string(), "http://127.0.0.1:7890".to_string()),
                ("ALL_PROXY".to_string(), "http://127.0.0.1:7890".to_string()),
            ]
        );
    }

    #[test]
    fn ignores_proxy_envs_when_not_in_manual_proxy_mode() {
        let envs = proxy_env_vars(&InstallNetworkConfig {
            mode: InstallNetworkMode::SystemProxy,
            proxy_url: Some("http://127.0.0.1:7890".to_string()),
            npm_registry: NpmRegistryOption::Default,
            custom_npm_registry: None,
        });

        assert!(envs.is_empty());
    }
}
