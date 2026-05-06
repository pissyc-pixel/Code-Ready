import type { ToolInstallStatus, ToolStatus } from "../types/tool";

export const statusMeta: Record<
  ToolInstallStatus,
  { label: string; className: string }
> = {
  checking: { label: "检测中", className: "checking" },
  installed: { label: "已安装", className: "installed" },
  missing: { label: "未安装", className: "missing" },
  installing: { label: "安装中", className: "installing" },
  install_failed: { label: "安装失败", className: "install-failed" },
  detect_failed: { label: "检测失败", className: "detect-failed" },
  installed_but_path_missing: {
    label: "PATH 缺失",
    className: "path-missing",
  },
  broken: { label: "已损坏", className: "broken" },
};

export function getToolVersionLabel(
  tool?: Pick<ToolStatus, "id" | "version" | "executablePath">,
): string {
  if (tool?.version) {
    return tool.version;
  }

  if (tool?.id === "ccswitch" && tool.executablePath) {
    return "版本未提供";
  }

  return "—";
}
