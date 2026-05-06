/// <reference types="vitest" />

import { act, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import type { Event } from "@tauri-apps/api/event";
import type { AppConfig } from "./types/config";
import type {
  DetectResultEvent,
  InstallProgressEvent,
  InstallStatusEvent,
} from "./types/events";
import type { ToolStatus } from "./types/tool";

const detectAllToolsMock = vi.fn<() => Promise<ToolStatus[]>>();
const detectToolMock = vi.fn();
const installToolMock = vi.fn<() => Promise<void>>();
const reinstallToolMock = vi.fn<() => Promise<void>>();
const installLatestToolMock = vi.fn<() => Promise<void>>();
const cancelInstallMock = vi.fn<() => Promise<void>>();
const getConfigMock = vi.fn<() => Promise<AppConfig>>();
const updateConfigMock = vi.fn();
const isAdminMock = vi.fn<() => Promise<boolean>>();
const openCcSwitchMock = vi.fn<() => Promise<void>>();
const openSubscriptionPageMock = vi.fn<() => Promise<void>>();
const restartAsAdminMock = vi.fn<() => Promise<void>>();
const getLogPreviewMock = vi.fn<() => Promise<string[]>>();
const openFullLogFileMock = vi.fn<() => Promise<void>>();
const openLogDirectoryMock = vi.fn<() => Promise<void>>();
const exportDiagnosticsLogZipMock = vi.fn<() => Promise<{ path: string }>>();
const listenMock = vi.fn();
const clipboardWriteTextMock = vi.fn<() => Promise<void>>();

vi.mock("./lib/api", () => ({
  detectAllTools: () => detectAllToolsMock(),
  detectTool: (...args: unknown[]) => detectToolMock(...args),
  installTool: (...args: unknown[]) => installToolMock(...args),
  reinstallTool: (...args: unknown[]) => reinstallToolMock(...args),
  installLatestTool: (...args: unknown[]) => installLatestToolMock(...args),
  cancelInstall: (...args: unknown[]) => cancelInstallMock(...args),
  getConfig: () => getConfigMock(),
  updateConfig: (...args: unknown[]) => updateConfigMock(...args),
  isAdmin: () => isAdminMock(),
  openCcSwitch: () => openCcSwitchMock(),
  openSubscriptionPage: () => openSubscriptionPageMock(),
  restartAsAdmin: () => restartAsAdminMock(),
  getLogPreview: (...args: unknown[]) => getLogPreviewMock(...args),
  openFullLogFile: () => openFullLogFileMock(),
  openLogDirectory: () => openLogDirectoryMock(),
  exportDiagnosticsLogZip: () => exportDiagnosticsLogZipMock(),
}));

let detectResultHandler: ((event: Event<DetectResultEvent>) => void) | null = null;
let installStatusHandler: ((event: Event<InstallStatusEvent>) => void) | null = null;
let installProgressHandler: ((event: Event<InstallProgressEvent>) => void) | null = null;

vi.mock("@tauri-apps/api/event", () => ({
  listen: (eventName: string, handler: (event: Event<unknown>) => void) => {
    listenMock(eventName);
    if (eventName === "detect:result") {
      detectResultHandler = handler as (event: Event<DetectResultEvent>) => void;
    }
    if (eventName === "install:status") {
      installStatusHandler = handler as (event: Event<InstallStatusEvent>) => void;
    }
    if (eventName === "install:progress") {
      installProgressHandler = handler as (event: Event<InstallProgressEvent>) => void;
    }
    return Promise.resolve(() => Promise.resolve());
  },
}));

import App from "./App";

function makeTool(
  overrides: Partial<ToolStatus> & Pick<ToolStatus, "id" | "name" | "category">,
): ToolStatus {
  return {
    status: "installed",
    version: "1.0.0",
    executablePath: "C:\\Tools\\tool.exe",
    detectionMethod: "combined",
    lastCheckedAt: "2026-05-06T16:00:00+08:00",
    ...overrides,
  };
}

describe("App UI migration", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    detectResultHandler = null;
    installStatusHandler = null;
    installProgressHandler = null;
    clipboardWriteTextMock.mockResolvedValue(undefined);
    Object.defineProperty(navigator, "clipboard", {
      configurable: true,
      value: { writeText: clipboardWriteTextMock },
    });
    getConfigMock.mockResolvedValue({
      installNetwork: {
        mode: "none",
        npmRegistry: "default",
      },
      ccswitchPath: undefined,
      ccswitchDownloadSources: [{ name: "mirror-a", url: "https://a", priority: 1, enabled: true, kind: "direct_exe" }],
      subscriptionPageUrl: "https://nodes.example.com/dashboard",
    });
    updateConfigMock.mockImplementation(async (patch: Partial<AppConfig>) => ({
      installNetwork: {
        mode: "none",
        npmRegistry: "default",
        ...(patch.installNetwork ?? {}),
      },
      ccswitchPath: patch.ccswitchPath,
      ccswitchDownloadSources: [{ name: "mirror-a", url: "https://a", priority: 1, enabled: true, kind: "direct_exe" }],
      subscriptionPageUrl: patch.subscriptionPageUrl ?? "https://nodes.example.com/dashboard",
    }));
    isAdminMock.mockResolvedValue(true);
    getLogPreviewMock.mockResolvedValue(["existing log line"]);
    openFullLogFileMock.mockResolvedValue(undefined);
    openLogDirectoryMock.mockResolvedValue(undefined);
    exportDiagnosticsLogZipMock.mockResolvedValue({ path: "C:\\logs\\diagnostics.zip" });
  });

  it("renders the app shell and dashboard summaries from real tool state", async () => {
    detectAllToolsMock.mockResolvedValue([
      makeTool({ id: "winget", name: "winget", category: "base" }),
      makeTool({ id: "git", name: "Git / Git Bash", category: "base" }),
      makeTool({ id: "node", name: "Node.js", category: "base" }),
      makeTool({
        id: "npm",
        name: "npm",
        category: "base",
        status: "installed_but_path_missing",
      }),
      makeTool({
        id: "python",
        name: "Python 3.11",
        category: "base",
        status: "detect_failed",
      }),
      makeTool({
        id: "claude",
        name: "Claude Code",
        category: "ai",
        status: "missing",
        version: undefined,
        executablePath: undefined,
      }),
      makeTool({ id: "codex", name: "Codex CLI", category: "ai" }),
      makeTool({ id: "opencode", name: "OpenCode", category: "ai", status: "broken" }),
      makeTool({
        id: "ccswitch",
        name: "ccSwitch",
        category: "ai",
        executablePath: "C:\\Tools\\ccswitch\\ccswitch.exe",
      }),
    ]);

    render(<App />);

    await waitFor(() => {
      expect(detectAllToolsMock).toHaveBeenCalledTimes(1);
      expect(getConfigMock).toHaveBeenCalledTimes(1);
      expect(isAdminMock).toHaveBeenCalledTimes(1);
      expect(listenMock).toHaveBeenCalledWith("detect:result");
      expect(listenMock).toHaveBeenCalledWith("install:status");
      expect(listenMock).toHaveBeenCalledWith("install:progress");
    });

    expect(screen.getByRole("heading", { name: "AI Coding 环境助手" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /^总览/ })).toHaveAttribute("aria-current", "page");
    expect(screen.getByText("已就绪")).toBeInTheDocument();
    expect(screen.getByText("需关注")).toBeInTheDocument();
    expect(screen.getByText("待安装")).toBeInTheDocument();
    expect(screen.getByText("OpenCode")).toBeInTheDocument();
    expect(screen.getByText("existing log line")).toBeInTheDocument();
  });

  it("keeps dashboard quick actions wired to real commands", async () => {
    detectAllToolsMock.mockResolvedValue([
      makeTool({
        id: "ccswitch",
        name: "ccSwitch",
        category: "ai",
        executablePath: "C:\\Tools\\ccswitch\\ccswitch.exe",
      }),
    ]);
    openCcSwitchMock.mockResolvedValue(undefined);
    openSubscriptionPageMock.mockResolvedValue(undefined);

    render(<App />);

    await screen.findByRole("button", { name: "重新检测全部" });

    fireEvent.click(screen.getByRole("button", { name: "打开 ccSwitch" }));
    fireEvent.click(screen.getByRole("button", { name: "打开订阅页" }));
    fireEvent.click(screen.getByRole("button", { name: "导出诊断 zip" }));
    fireEvent.click(screen.getByRole("button", { name: "重新检测全部" }));

    await waitFor(() => {
      expect(openCcSwitchMock).toHaveBeenCalledTimes(1);
      expect(openSubscriptionPageMock).toHaveBeenCalledTimes(1);
      expect(exportDiagnosticsLogZipMock).toHaveBeenCalledTimes(1);
      expect(detectAllToolsMock).toHaveBeenCalledTimes(2);
    });

    expect(
      screen.getByText("诊断 zip 已导出：C:\\logs\\diagnostics.zip"),
    ).toBeInTheDocument();
  });

  it("renders the migrated environment page with real base tool actions and path repair", async () => {
    detectAllToolsMock.mockResolvedValue([
      makeTool({
        id: "git",
        name: "Git / Git Bash",
        category: "base",
        status: "missing",
        version: undefined,
        executablePath: undefined,
      }),
      makeTool({
        id: "node",
        name: "Node.js",
        category: "base",
        status: "installed",
        version: "v24.1.0",
        executablePath: "C:\\Program Files\\nodejs\\node.exe",
      }),
      makeTool({
        id: "npm",
        name: "npm",
        category: "base",
        status: "installed_but_path_missing",
        version: "11.3.0",
        executablePath: "%APPDATA%\\npm\\npm.cmd",
      }),
      makeTool({
        id: "python",
        name: "Python 3.11",
        category: "base",
        status: "detect_failed",
        version: undefined,
        executablePath: undefined,
        errorMessage: "检测命令超时（5s）。可能受网络或杀毒软件影响。",
      }),
    ]);
    detectToolMock.mockResolvedValue(
      makeTool({
        id: "git",
        name: "Git / Git Bash",
        category: "base",
      }),
    );

    render(<App />);

    fireEvent.click(await screen.findByRole("button", { name: /^基础环境/ }));

    const envRegion = await screen.findByRole("region", { name: "基础环境" });
    expect(within(envRegion).getByRole("heading", { name: "基础环境" })).toBeInTheDocument();
    expect(
      screen.getByText(/Git \/ Node \/ npm \/ Python 是 AI Coding CLI 的前置依赖/),
    ).toBeInTheDocument();
    expect(screen.getByText("版本管理")).toBeInTheDocument();
    expect(screen.getByText("包管理器")).toBeInTheDocument();
    expect(screen.getByText("检测说明")).toBeInTheDocument();

    fireEvent.click(screen.getAllByRole("button", { name: "查看 PATH 修复说明" })[0]);
    await screen.findByRole("dialog", { name: "可执行文件存在，但终端 PATH 没刷新" });

    const gitRow = screen.getByText("Git / Git Bash").closest("tr");
    expect(gitRow).not.toBeNull();
    fireEvent.click(within(gitRow as HTMLTableRowElement).getByRole("button", { name: "重新检测" }));

    await waitFor(() => {
      expect(detectToolMock).toHaveBeenCalledWith("git");
    });
  });

  it("renders the migrated ai tools page with real install actions and install context", async () => {
    detectAllToolsMock.mockResolvedValue([
      makeTool({
        id: "claude",
        name: "Claude Code",
        category: "ai",
        status: "missing",
        version: undefined,
        executablePath: undefined,
      }),
      makeTool({
        id: "codex",
        name: "Codex CLI",
        category: "ai",
        status: "checking",
        version: undefined,
        executablePath: undefined,
      }),
      makeTool({
        id: "opencode",
        name: "OpenCode",
        category: "ai",
        status: "broken",
        version: "0.9.2",
        executablePath: "C:\\Users\\dev\\AppData\\Roaming\\npm\\opencode.cmd",
        errorMessage: "命令存在，但版本探测失败。建议重新安装。",
      }),
      makeTool({
        id: "ccswitch",
        name: "ccSwitch",
        category: "ai",
        status: "installed",
        version: "0.4.1",
        executablePath: "C:\\Tools\\ccswitch\\ccswitch.exe",
      }),
    ]);
    openCcSwitchMock.mockResolvedValue(undefined);
    openSubscriptionPageMock.mockResolvedValue(undefined);

    render(<App />);

    fireEvent.click(await screen.findByRole("button", { name: /^AI 工具/ }));

    const aiRegion = await screen.findByRole("region", { name: "AI 工具" });
    expect(within(aiRegion).getByRole("heading", { name: "AI 工具" })).toBeInTheDocument();
    expect(
      screen.getByText(/Claude Code \/ Codex CLI \/ OpenCode \/ ccSwitch/),
    ).toBeInTheDocument();
    expect(screen.getAllByText("AI CLI").length).toBeGreaterThan(0);
    expect(screen.getByText("GUI · 节点切换")).toBeInTheDocument();
    expect(screen.getByText("安装说明")).toBeInTheDocument();
    expect(screen.getByText("npm install -g <package>")).toBeInTheDocument();
    expect(screen.getByText("不使用代理 (none)")).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "打开 ccSwitch" }));
    fireEvent.click(screen.getByRole("button", { name: "打开订阅页" }));

    await waitFor(() => {
      expect(openCcSwitchMock).toHaveBeenCalledTimes(1);
      expect(openSubscriptionPageMock).toHaveBeenCalledTimes(1);
    });
  });

  it("renders the migrated ccswitch page with real config actions", async () => {
    detectAllToolsMock.mockResolvedValue([
      makeTool({
        id: "ccswitch",
        name: "ccSwitch",
        category: "ai",
        version: "0.4.1",
        executablePath: "C:\\Tools\\ccswitch\\ccswitch.exe",
      }),
    ]);
    updateConfigMock.mockResolvedValue({
      installNetwork: {
        mode: "none",
        npmRegistry: "default",
      },
      ccswitchPath: "C:\\Program Files\\ccswitch\\ccswitch.exe",
      ccswitchDownloadSources: [{ name: "mirror-a", url: "https://a", priority: 1, enabled: true, kind: "direct_exe" }],
      subscriptionPageUrl: "https://nodes.example.com/dashboard",
    });
    detectToolMock.mockResolvedValue(
      makeTool({
        id: "ccswitch",
        name: "ccSwitch",
        category: "ai",
        version: "0.4.1",
        executablePath: "C:\\Program Files\\ccswitch\\ccswitch.exe",
      }),
    );

    render(<App />);

    fireEvent.click(await screen.findByRole("button", { name: /^ccSwitch/ }));
    const ccSwitchRegion = await screen.findByRole("region", { name: "ccSwitch" });
    expect(
      within(ccSwitchRegion).getByRole("heading", { name: "ccSwitch" }),
    ).toBeInTheDocument();

    const pathInput = screen.getByLabelText("可执行文件路径");
    fireEvent.change(pathInput, {
      target: { value: "C:\\Program Files\\ccswitch\\ccswitch.exe" },
    });
    fireEvent.click(screen.getAllByRole("button", { name: "保存" })[0]);

    await waitFor(() => {
      expect(updateConfigMock).toHaveBeenCalledWith({
        ccswitchPath: "C:\\Program Files\\ccswitch\\ccswitch.exe",
      });
      expect(detectToolMock).toHaveBeenCalledWith("ccswitch");
    });

    fireEvent.click(screen.getByRole("button", { name: "打开 ccSwitch" }));
    fireEvent.click(screen.getByRole("button", { name: "打开" }));

    await waitFor(() => {
      expect(openCcSwitchMock).toHaveBeenCalledTimes(1);
      expect(openSubscriptionPageMock).toHaveBeenCalledTimes(1);
    });
  });

  it("renders the migrated settings page with real config and admin actions", async () => {
    isAdminMock.mockResolvedValue(false);
    detectAllToolsMock.mockResolvedValue([]);
    updateConfigMock.mockResolvedValue({
      installNetwork: {
        mode: "manual_proxy",
        npmRegistry: "custom",
        proxyUrl: "http://127.0.0.1:7890",
        customNpmRegistry: "https://registry.example.com/",
      },
      ccswitchDownloadSources: [],
      subscriptionPageUrl: "https://nodes.example.com/dashboard",
    });
    restartAsAdminMock.mockResolvedValue(undefined);

    render(<App />);

    fireEvent.click(await screen.findByRole("button", { name: /^设置/ }));
    await screen.findByRole("heading", { name: "安装网络" });

    fireEvent.click(screen.getByRole("button", { name: /手动代理/ }));
    fireEvent.change(await screen.findByLabelText("代理 URL"), {
      target: { value: "http://127.0.0.1:7890" },
    });
    fireEvent.change(screen.getByLabelText("npm 源"), {
      target: { value: "custom" },
    });
    fireEvent.change(screen.getByLabelText("自定义源 URL"), {
      target: { value: "https://registry.example.com/" },
    });
    fireEvent.click(screen.getByRole("button", { name: "保存设置" }));

    await waitFor(() => {
      expect(updateConfigMock).toHaveBeenCalledWith({
        installNetwork: {
          mode: "manual_proxy",
          proxyUrl: "http://127.0.0.1:7890",
          npmRegistry: "custom",
          customNpmRegistry: "https://registry.example.com/",
        },
      });
    });

    fireEvent.click(screen.getByRole("button", { name: "以管理员身份重启" }));

    await waitFor(() => {
      expect(restartAsAdminMock).toHaveBeenCalledTimes(1);
    });
  });

  it("renders the migrated logs page with real preview actions and diagnostics context", async () => {
    detectAllToolsMock.mockResolvedValue([]);
    getLogPreviewMock.mockResolvedValue([
      "[2026-05-06T14:32:01.221+08:00] [stdout] Detect cycle started · 9 tools",
      "[2026-05-06T14:32:01.301+08:00] [stderr] npm: PATH does not contain %APPDATA%\\npm",
    ]);
    openFullLogFileMock.mockResolvedValue(undefined);
    openLogDirectoryMock.mockResolvedValue(undefined);
    exportDiagnosticsLogZipMock.mockResolvedValue({ path: "C:\\logs\\diagnostics.zip" });

    render(<App />);

    fireEvent.click(await screen.findByRole("button", { name: /^日志/ }));

    const logsRegion = await screen.findByRole("region", { name: "日志" });
    expect(within(logsRegion).getByRole("heading", { name: "日志" })).toBeInTheDocument();
    expect(screen.getByText("按级别与来源自动着色")).toBeInTheDocument();
    expect(screen.getByText("诊断 zip 包含什么")).toBeInTheDocument();
    expect(screen.getByText("detect.log")).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "刷新预览" }));
    fireEvent.click(screen.getByRole("button", { name: "打开完整日志" }));
    fireEvent.click(screen.getByRole("button", { name: "打开日志目录" }));
    fireEvent.click(screen.getByRole("button", { name: "导出诊断 zip" }));

    await waitFor(() => {
      expect(getLogPreviewMock).toHaveBeenCalledTimes(2);
      expect(openFullLogFileMock).toHaveBeenCalledTimes(1);
      expect(openLogDirectoryMock).toHaveBeenCalledTimes(1);
      expect(exportDiagnosticsLogZipMock).toHaveBeenCalledTimes(1);
    });
  });

  it("surfaces path, detect, and install failure states with real actions", async () => {
    detectAllToolsMock.mockResolvedValue([
      makeTool({
        id: "npm",
        name: "npm",
        category: "base",
        status: "installed_but_path_missing",
        version: "11.3.0",
        executablePath: "%APPDATA%\\npm\\npm.cmd",
      }),
      makeTool({
        id: "python",
        name: "Python 3.11",
        category: "base",
        status: "detect_failed",
        version: undefined,
        executablePath: undefined,
        errorMessage: "检测命令超时（5s）。可能受网络或杀毒软件影响。",
      }),
      makeTool({
        id: "opencode",
        name: "OpenCode",
        category: "ai",
        status: "broken",
        version: "0.9.2",
        executablePath: "C:\\Users\\dev\\AppData\\Roaming\\npm\\opencode.cmd",
        errorMessage: "命令存在，但版本探测失败。建议重新安装。",
      }),
    ]);
    detectToolMock.mockResolvedValue(
      makeTool({
        id: "python",
        name: "Python 3.11",
        category: "base",
        status: "installed",
        executablePath: "C:\\Python311\\python.exe",
      }),
    );
    reinstallToolMock.mockResolvedValue(undefined);
    openLogDirectoryMock.mockResolvedValue(undefined);

    render(<App />);

    fireEvent.click(await screen.findByRole("button", { name: /^基础环境/ }));
    expect((await screen.findAllByText("PATH 缺失")).length).toBeGreaterThan(0);
    expect((await screen.findAllByText("检测失败")).length).toBeGreaterThan(0);

    fireEvent.click(screen.getByRole("button", { name: "重新检测 Python 3.11" }));
    fireEvent.click(screen.getAllByRole("button", { name: "打开日志目录" })[0]);

    await waitFor(() => {
      expect(detectToolMock).toHaveBeenCalledWith("python");
      expect(openLogDirectoryMock).toHaveBeenCalledTimes(1);
    });

    fireEvent.click(screen.getByRole("button", { name: /^AI 工具/ }));
    expect(await screen.findByText("安装失败 / 已损坏")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "重新安装 OpenCode" }));

    await waitFor(() => {
      expect(reinstallToolMock).toHaveBeenCalledWith("opencode");
    });
  });

  it("shows admin and install activity states from real runtime events", async () => {
    isAdminMock.mockResolvedValue(false);
    detectAllToolsMock.mockResolvedValue([
      makeTool({
        id: "codex",
        name: "Codex CLI",
        category: "ai",
        status: "missing",
        version: undefined,
        executablePath: undefined,
      }),
    ]);
    restartAsAdminMock.mockResolvedValue(undefined);
    cancelInstallMock.mockResolvedValue(undefined);

    render(<App />);

    await screen.findByRole("button", { name: /^总览/ });

    await act(async () => {
      installStatusHandler?.({
        event: "install:status",
        id: 2,
        payload: {
          toolId: "codex",
          status: "installing",
          phase: "running",
          timestamp: "2026-05-06T16:31:00+08:00",
        },
        windowLabel: "main",
      });
    });

    expect(await screen.findByText("当前为标准用户运行")).toBeInTheDocument();
    expect(screen.getByText("正在安装 Codex CLI")).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "取消安装" }));
    fireEvent.click(screen.getByRole("button", { name: "以管理员身份重启" }));

    await waitFor(() => {
      expect(cancelInstallMock).toHaveBeenCalledTimes(1);
      expect(restartAsAdminMock).toHaveBeenCalledTimes(1);
    });
  });

  it("streams install logs into the dashboard preview", async () => {
    detectAllToolsMock.mockResolvedValue([]);

    render(<App />);

    await screen.findByText("existing log line");

    await act(async () => {
      installProgressHandler?.({
        event: "install:progress",
        id: 1,
        payload: {
          toolId: "codex",
          phase: "running",
          stream: "stdout",
          line: "progress line 1",
          timestamp: "2026-05-06T16:30:00+08:00",
        },
        windowLabel: "main",
      });
    });

    await waitFor(() => {
      expect(screen.getByText(/\[stdout\] progress line 1/)).toBeInTheDocument();
    });
  });
});
