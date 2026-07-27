export const messages = {
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

export type MessageKey = keyof typeof messages;
