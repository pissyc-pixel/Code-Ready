export type ToolCategory = "base" | "ai";

export type ToolInstallStatus =
  | "checking"
  | "installed"
  | "missing"
  | "installing"
  | "install_failed"
  | "detect_failed"
  | "installed_but_path_missing"
  | "broken";

export type DetectionMethod =
  | "which"
  | "version_flag"
  | "path_probe"
  | "registry"
  | "npm_global_probe"
  | "combined";

export type ToolId =
  | "winget"
  | "git"
  | "node"
  | "npm"
  | "python"
  | "claude"
  | "codex"
  | "opencode"
  | "ccswitch";

export type ToolStatus = {
  id: ToolId;
  name: string;
  category: ToolCategory;
  status: ToolInstallStatus;
  version?: string;
  executablePath?: string;
  detectionMethod: DetectionMethod;
  errorMessage?: string;
  suggestion?: string;
  lastCheckedAt: string;
};

export function isInstalled(tool: ToolStatus): boolean {
  return (
    tool.status === "installed" ||
    tool.status === "installed_but_path_missing"
  );
}
