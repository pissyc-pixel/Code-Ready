export const messages = {
  "app.title": "Code-Ready V2",
  "privacy.zeroTelemetry": "检测只在当前设备上进行，不上传遥测或命令输出。",
  "onboarding.welcome.title": "先看看这台电脑的开发环境",
  "onboarding.welcome.body":
    "Code-Ready 会只读检查 Git、Claude Code 和 Codex CLI，不会安装、升级或修复任何工具。",
  "onboarding.start": "开始检测",
  "onboarding.detecting.title": "正在检测开发工具",
  "onboarding.explain.title": "检测完成",
  "onboarding.enterStatusCenter": "进入状态中心",
  "dashboard.title": "开发环境状态",
  "dashboard.readOnly": "只读检测",
  "dashboard.needsAttention": "需要处理",
  "dashboard.healthy": "正常可用",
  "dashboard.notYetDetected": "尚未检测",
  "dashboard.path": "检测路径",
  "detection.running": "检测正在进行…",
  "detection.runFailed": "检测任务意外停止，工具卡片仍显示最近一次已知事实。",
  "detection.startError": "暂时无法开始检测，请重试。",
  "sync.eventUnavailable": "实时更新暂不可用；重新聚焦窗口或手动刷新可读取最新状态。",
  "sync.refreshFailed": "暂时无法刷新，下面仍显示最近一次已知状态。",
  "status.absent": "未检测到",
  "status.presentHealthy": "可以正常运行",
  "status.presentPathIssue": "已安装，但当前环境路径没有指向它",
  "status.presentBroken": "已找到，但无法正常完成版本检查",
  "status.unknown": "暂时无法判断",
  "version.current": "版本符合当前基线",
  "version.outdated": "版本低于支持基线",
  "version.newerThanKnown": "版本高于当前已知范围",
  "version.notComparable": "已检测到版本，但本版本暂不判断新旧",
  "version.unknown": "未能读取版本",
  "tools.git.name": "Git",
  "tools.claudeCode.name": "Claude Code",
  "tools.codexCli.name": "Codex CLI",
  "actions.retry": "重试",
  "actions.refresh": "刷新状态",
  "actions.redetect": "重新检测",
  "actions.redetectAll": "重新检测全部",
  "actions.redetectOne": "重新检测此工具",

  // Slice 0 compatibility labels retained until the status center replaces
  // the temporary summary page.
  appTitle: "Code-Ready V2",
  baseline: "重写基线",
  loadingBootstrap: "正在读取当前设备…",
  bootstrapError: "暂时无法读取设备状态。",
  platformHeading: "当前支持平台",
  platformWindowsX64: "Windows 11 x64",
  platformMacosArm64: "macOS Apple Silicon",
  platformUnknown: "未识别的平台",
  registryHeading: "内置工具定义",
  toolCount: (count: number) => `已载入 ${count} 项内置工具定义`,
  nextSlicePlaceholder: "检测与引导安装将在下一切片接入。",
} as const;

export type MessageKey = Exclude<keyof typeof messages, "toolCount">;

export function message(key: MessageKey): string {
  const value = messages[key];
  return typeof value === "string" ? value : "未知消息";
}

const TOOL_LABEL_KEYS: Record<string, MessageKey> = {
  "tools.git.name": "tools.git.name",
  "tools.claudeCode.name": "tools.claudeCode.name",
  "tools.codexCli.name": "tools.codexCli.name",
};

export function toolLabel(labelKey: string): string {
  const messageKey = TOOL_LABEL_KEYS[labelKey];
  return messageKey === undefined ? "未知工具" : message(messageKey);
}

export function formatObservedVersion(version: string): string {
  return `版本 ${version}`;
}

export function formatRedetectOne(label: string): string {
  return `${messages["actions.redetect"]} ${label}`;
}
