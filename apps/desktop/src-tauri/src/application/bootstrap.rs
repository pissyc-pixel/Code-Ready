use std::sync::Arc;

use crate::application::snapshot_store::SnapshotStore;
use crate::domain::contracts::AppSnapshot;

pub struct BootstrapService {
    store: Arc<SnapshotStore>,
}

impl BootstrapService {
    pub fn new(store: Arc<SnapshotStore>) -> Self {
        Self { store }
    }

    pub fn get_snapshot(&self) -> AppSnapshot {
        self.store.snapshot()
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

    struct FixedClock;

    impl crate::tools::shared::Clock for FixedClock {
        fn now_epoch_ms(&self) -> u64 {
            100
        }
    }

    #[test]
    fn bootstrap_returns_the_current_snapshot_without_advancing_versions() {
        let clock = Arc::new(FixedClock);
        let store = Arc::new(crate::application::snapshot_store::SnapshotStore::new(
            PlatformId::MacosArm64,
            built_in_tool_registry(),
            clock,
        ));
        let service = BootstrapService::new(store.clone());
        let before = store.snapshot();

        assert_eq!(service.get_snapshot(), before);
        assert_eq!(service.get_snapshot(), before);
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
