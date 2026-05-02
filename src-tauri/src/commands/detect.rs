use crate::detector::{self, ToolStatus};

#[tauri::command]
pub async fn detect_tool(
    app: tauri::AppHandle,
    tool_id: String,
) -> Result<ToolStatus, String> {
    detector::detect_tool(app, &tool_id).await
}

#[tauri::command]
pub async fn detect_all_tools(
    app: tauri::AppHandle,
) -> Result<Vec<ToolStatus>, String> {
    detector::detect_all_tools(app).await
}
