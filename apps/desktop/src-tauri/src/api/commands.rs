use crate::application::bootstrap::BootstrapService;
use crate::domain::contracts::BootstrapState;

fn get_bootstrap_state_inner(service: &BootstrapService) -> BootstrapState {
    service.get_state()
}

#[tauri::command]
pub fn get_bootstrap_state(service: tauri::State<'_, BootstrapService>) -> BootstrapState {
    get_bootstrap_state_inner(&service)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::application::bootstrap::BootstrapService;
    use crate::domain::contracts::PlatformId;
    use crate::domain::tool_registry::built_in_tool_registry;
    use crate::platform::fake::FakePlatformAdapter;

    #[test]
    fn get_bootstrap_state_uses_the_bootstrap_service() {
        let adapter = Arc::new(FakePlatformAdapter::new(PlatformId::WindowsX64));
        let service = BootstrapService::new(adapter);
        let state = super::get_bootstrap_state_inner(&service);

        assert_eq!(state.schema_version, 1);
        assert_eq!(state.platform, PlatformId::WindowsX64);
        assert_eq!(state.tools.len(), 5);
        assert_eq!(state.tools, built_in_tool_registry());
    }
}
