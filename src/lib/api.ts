import { invoke } from "@tauri-apps/api/core";
import type { AppConfig, AppConfigPatch } from "../types/config";
import type { ToolId, ToolStatus } from "../types/tool";

export type LogPreview = {
  lines: string[];
  totalLines: number;
  truncated: boolean;
  logDir: string;
};

export type ExportLogsResult = {
  zipPath: string;
  fileCount: number;
};

export type DiagnosticsExportResult = {
  path: string;
};

export async function detectTool(toolId: ToolId): Promise<ToolStatus> {
  return invoke<ToolStatus>("detect_tool", { toolId });
}

export async function detectAllTools(): Promise<ToolStatus[]> {
  return invoke<ToolStatus[]>("detect_all_tools");
}

export async function installTool(toolId: ToolId): Promise<void> {
  return invoke<void>("install_tool", { toolId });
}

export async function reinstallTool(toolId: ToolId): Promise<void> {
  return invoke<void>("reinstall_tool", { toolId });
}

export async function installLatestTool(toolId: ToolId): Promise<void> {
  return invoke<void>("install_latest_tool", { toolId });
}

export async function cancelInstall(): Promise<void> {
  return invoke<void>("cancel_install");
}

export async function getConfig(): Promise<AppConfig> {
  return invoke<AppConfig>("get_config");
}

export async function updateConfig(patch: AppConfigPatch): Promise<AppConfig> {
  return invoke<AppConfig>("update_config", { patch });
}

export async function resetConfig(): Promise<AppConfig> {
  return invoke<AppConfig>("reset_config");
}

export async function openCcSwitch(): Promise<void> {
  return invoke<void>("open_ccswitch");
}

export async function openSubscriptionPage(): Promise<void> {
  return invoke<void>("open_subscription_page");
}

export async function getLogPreview(limit = 5000): Promise<string[]> {
  const preview = await invoke<LogPreview>("get_log_preview", { limit });
  return preview.lines;
}

export async function openFullLogFile(): Promise<void> {
  return invoke<void>("open_full_log_file");
}

export async function openLogDirectory(): Promise<void> {
  return invoke<void>("open_log_directory");
}

export async function exportDiagnosticsLogZip(): Promise<DiagnosticsExportResult> {
  const result = await invoke<ExportLogsResult>("export_logs");
  return { path: result.zipPath };
}

export async function isAdmin(): Promise<boolean> {
  return invoke<boolean>("is_admin");
}

export async function restartAsAdmin(): Promise<void> {
  return invoke<void>("restart_as_admin");
}
