mod commands;
mod detector;
mod installer;
mod state;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(state::AppState::default())
        .invoke_handler(tauri::generate_handler![
            commands::detect::detect_tool,
            commands::detect::detect_all_tools,
            commands::install::install_tool,
            commands::install::cancel_install
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
