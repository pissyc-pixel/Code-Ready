mod commands;
mod config;
mod detector;
mod installer;
mod logger;
mod network;
mod process;
mod state;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(state::AppState::default())
        .invoke_handler(tauri::generate_handler![
            commands::config::get_config,
            commands::config::update_config,
            commands::config::reset_config,
            commands::ccswitch::open_ccswitch,
            commands::detect::detect_tool,
            commands::detect::detect_all_tools,
            commands::install::install_tool,
            commands::install::reinstall_tool,
            commands::install::install_latest_tool,
            commands::install::cancel_install,
            commands::privilege::is_admin,
            commands::privilege::restart_as_admin
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
