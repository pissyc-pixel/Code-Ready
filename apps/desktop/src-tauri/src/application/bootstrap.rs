use std::sync::Arc;

use crate::domain::contracts::BootstrapState;
use crate::domain::tool_registry::built_in_tool_registry;
use crate::platform::PlatformAdapter;

pub struct BootstrapService {
    adapter: Arc<dyn PlatformAdapter>,
}

impl BootstrapService {
    pub fn new(adapter: Arc<dyn PlatformAdapter>) -> Self {
        Self { adapter }
    }

    pub fn get_state(&self) -> BootstrapState {
        BootstrapState {
            schema_version: 1,
            platform: self.adapter.platform(),
            tools: built_in_tool_registry(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::Arc;

    use super::BootstrapService;
    use crate::domain::contracts::PlatformId;
    use crate::domain::tool_registry::built_in_tool_registry;
    use crate::platform::fake::FakePlatformAdapter;

    #[test]
    fn bootstrap_uses_platform_adapter_and_registry() {
        let adapter = Arc::new(FakePlatformAdapter::new(PlatformId::MacosArm64));
        let service = BootstrapService::new(adapter);

        let state = service.get_state();

        assert_eq!(state.schema_version, 1);
        assert_eq!(state.platform, PlatformId::MacosArm64);
        assert_eq!(state.tools, built_in_tool_registry());
    }

    #[test]
    fn bootstrap_can_use_the_windows_fake_without_a_compile_time_platform_branch() {
        let adapter = Arc::new(FakePlatformAdapter::new(PlatformId::WindowsX64));
        let service = BootstrapService::new(adapter);

        let state = service.get_state();

        assert_eq!(state.schema_version, 1);
        assert_eq!(state.platform, PlatformId::WindowsX64);
        assert_eq!(state.tools, built_in_tool_registry());
    }

    #[test]
    fn domain_and_application_sources_do_not_cross_platform_or_tauri_boundaries() {
        let source_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
        let forbidden = [
            ["std", "process"].join("::"),
            ["std", "env"].join("::"),
            ["std", "fs"].join("::"),
            "power".to_owned() + "shell",
            ["cmd", ".", "exe"].concat(),
            ["/bin", "/sh"].concat(),
            ["c", "fg", "(", "target_os"].concat(),
            "USERPROFILE".to_owned(),
            "LOCALAPPDATA".to_owned(),
            "tauri".to_owned() + "::",
        ];

        for module in ["domain", "application"] {
            assert_source_tree_has_no_forbidden_tokens(&source_root.join(module), &forbidden);
        }
    }

    fn assert_source_tree_has_no_forbidden_tokens(directory: &Path, forbidden: &[String]) {
        let mut pending = vec![directory.to_path_buf()];

        while let Some(path) = pending.pop() {
            for entry in fs::read_dir(path).expect("source directory is readable") {
                let entry = entry.expect("source entry is readable");
                let entry_path = entry.path();

                if entry_path.is_dir() {
                    pending.push(entry_path);
                    continue;
                }

                if entry_path
                    .extension()
                    .and_then(|extension| extension.to_str())
                    != Some("rs")
                {
                    continue;
                }

                let source = fs::read_to_string(&entry_path).expect("source file is readable");
                let source = source
                    .split_once("#[cfg(test)]")
                    .map_or(source.as_str(), |(production, _)| production);
                for token in forbidden {
                    assert!(
                        !source.contains(token),
                        "{} contains forbidden token {token}",
                        entry_path.display()
                    );
                }
            }
        }
    }
}
