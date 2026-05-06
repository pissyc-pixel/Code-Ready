/// <reference types="vitest" />

import { act, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import type { Event } from "@tauri-apps/api/event";
import type { AppConfig } from "./types/config";
import type { DetectResultEvent, InstallProgressEvent, InstallStatusEvent } from "./types/events";
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

let detectResultHandler:
  | ((event: Event<DetectResultEvent>) => void)
  | null = null;
let installStatusHandler:
  | ((event: Event<InstallStatusEvent>) => void)
  | null = null;
let installProgressHandler:
  | ((event: Event<InstallProgressEvent>) => void)
  | null = null;

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

function makeTool(overrides: Partial<ToolStatus> & Pick<ToolStatus, "id" | "name" | "category">): ToolStatus {
  return {
    status: "installed",
    version: "1.0.0",
    executablePath: "C:\\Tools\\tool.exe",
    detectionMethod: "combined",
    lastCheckedAt: "2026-05-06T16:00:00+08:00",
    ...overrides,
  };
}

describe("App shell migration", () => {
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
      ccswitchDownloadSources: [],
      subscriptionPageUrl: "https://nodes.example.com/dashboard",
    });
    isAdminMock.mockResolvedValue(true);
    getLogPreviewMock.mockResolvedValue(["existing log line"]);
    openFullLogFileMock.mockResolvedValue(undefined);
    openLogDirectoryMock.mockResolvedValue(undefined);
    exportDiagnosticsLogZipMock.mockResolvedValue({ path: "C:\\logs\\diagnostics.zip" });
  });

  it("renders the migrated app shell and dashboard summaries from real tool state", async () => {
    detectAllToolsMock.mockResolvedValue([
      makeTool({ id: "winget", name: "winget", category: "base" }),
      makeTool({ id: "git", name: "Git / Git Bash", category: "base" }),
      makeTool({ id: "node", name: "Node.js", category: "base" }),
      makeTool({ id: "npm", name: "npm", category: "base", status: "installed_but_path_missing" }),
      makeTool({ id: "python", name: "Python 3.11", category: "base", status: "detect_failed" }),
      makeTool({ id: "claude", name: "Claude Code", category: "ai", status: "missing", version: undefined, executablePath: undefined }),
      makeTool({ id: "codex", name: "Codex CLI", category: "ai" }),
      makeTool({ id: "opencode", name: "OpenCode", category: "ai", status: "broken" }),
      makeTool({ id: "ccswitch", name: "ccSwitch", category: "ai", executablePath: "C:\\Tools\\ccswitch\\ccswitch.exe" }),
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
      makeTool({ id: "ccswitch", name: "ccSwitch", category: "ai", executablePath: "C:\\Tools\\ccswitch\\ccswitch.exe" }),
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

    expect(screen.getByText("Diagnostics zip exported: C:\\logs\\diagnostics.zip")).toBeInTheDocument();
  });

  it("shows the non-admin banner and keeps the restart action", async () => {
    isAdminMock.mockResolvedValue(false);
    detectAllToolsMock.mockResolvedValue([]);
    restartAsAdminMock.mockResolvedValue(undefined);

    render(<App />);

    fireEvent.click(await screen.findByRole("button", { name: /^设置/ }));
    await screen.findByText("当前为标准用户运行");
    fireEvent.click(screen.getByRole("button", { name: "以管理员身份重启" }));

    await waitFor(() => {
      expect(restartAsAdminMock).toHaveBeenCalledTimes(1);
    });
  });

  it("preserves legacy tool operations on the environment page inside the new shell", async () => {
    detectAllToolsMock.mockResolvedValue([
      makeTool({
        id: "git",
        name: "Git / Git Bash",
        category: "base",
        status: "missing",
        version: undefined,
        executablePath: undefined,
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

    const section = await screen.findByRole("region", { name: "基础环境" });
    fireEvent.click(within(section).getByRole("button", { name: "重新检测" }));

    await waitFor(() => {
      expect(detectToolMock).toHaveBeenCalledWith("git");
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
