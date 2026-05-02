import { invoke } from "@tauri-apps/api/core";
import type { ToolId, ToolStatus } from "../types/tool";

export async function detectTool(toolId: ToolId): Promise<ToolStatus> {
  return invoke<ToolStatus>("detect_tool", { toolId });
}

export async function detectAllTools(): Promise<ToolStatus[]> {
  return invoke<ToolStatus[]>("detect_all_tools");
}

export async function installTool(toolId: ToolId): Promise<void> {
  return invoke<void>("install_tool", { toolId });
}

export async function cancelInstall(): Promise<void> {
  return invoke<void>("cancel_install");
}
