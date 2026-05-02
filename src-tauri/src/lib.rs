mod commands;
mod detector;
mod state;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(state::AppState)
        .invoke_handler(tauri::generate_handler![
            commands::detect::detect_tool,
            commands::detect::detect_all_tools
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
