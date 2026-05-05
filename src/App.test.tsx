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
const resetConfigMock = vi.fn();
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
const installButtonName = /^(安装|瀹夎.*)$/;
const cancelInstallButtonName = /^(取消安装|鍙栨秷瀹夎.*)$/;

vi.mock("./lib/api", () => ({
  detectAllTools: () => detectAllToolsMock(),
  detectTool: (...args: unknown[]) => detectToolMock(...args),
  installTool: (...args: unknown[]) => installToolMock(...args),
  reinstallTool: (...args: unknown[]) => reinstallToolMock(...args),
  installLatestTool: (...args: unknown[]) => installLatestToolMock(...args),
  cancelInstall: (...args: unknown[]) => cancelInstallMock(...args),
  getConfig: () => getConfigMock(),
  updateConfig: (...args: unknown[]) => updateConfigMock(...args),
  resetConfig: () => resetConfigMock(),
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
let installProgressSubscribed = false;

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
      installProgressSubscribed = true;
      installProgressHandler = handler as (event: Event<InstallProgressEvent>) => void;
    }
    return Promise.resolve(() => Promise.resolve());
  },
}));

import App from "./App";

describe("App", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    detectResultHandler = null;
    installStatusHandler = null;
    installProgressHandler = null;
    installProgressSubscribed = false;
    clipboardWriteTextMock.mockResolvedValue(undefined);
    Object.defineProperty(navigator, "clipboard", {
      configurable: true,
      value: {
        writeText: clipboardWriteTextMock,
      },
    });
    getConfigMock.mockResolvedValue({
      installNetwork: {
        mode: "none",
        npmRegistry: "default",
      },
      ccswitchPath: undefined,
      ccswitchDownloadSources: [],
      subscriptionPageUrl: undefined,
    });
    isAdminMock.mockResolvedValue(true);
    getLogPreviewMock.mockResolvedValue([]);
    openFullLogFileMock.mockResolvedValue(undefined);
    openLogDirectoryMock.mockResolvedValue(undefined);
    exportDiagnosticsLogZipMock.mockResolvedValue({ path: "C:\\logs\\diagnostics.zip" });
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("starts detect_all_tools on mount and applies detect:result updates", async () => {
    detectAllToolsMock.mockResolvedValue([
      {
        id: "winget",
        name: "winget",
        category: "base",
        status: "installed",
        version: "v-test-final",
        executablePath: "C:\\final\\winget.exe",
        detectionMethod: "which",
        lastCheckedAt: "2026-05-02T16:30:00+08:00",
      },
    ]);

    render(<App />);

    await waitFor(() => {
      expect(detectAllToolsMock).toHaveBeenCalledTimes(1);
      expect(listenMock).toHaveBeenCalledWith("detect:result");
      expect(listenMock).toHaveBeenCalledWith("install:status");
      expect(installProgressSubscribed).toBe(true);
      expect(getConfigMock).toHaveBeenCalledTimes(1);
      expect(isAdminMock).toHaveBeenCalledTimes(1);
    });

    await act(async () => {
      detectResultHandler?.({
        event: "detect:result",
        id: 1,
        payload: {
          toolId: "winget",
          result: {
            id: "winget",
            name: "winget",
            category: "base",
            status: "installed",
            version: "v-test-event",
            executablePath: "C:\\event\\winget.exe",
            detectionMethod: "which",
            lastCheckedAt: "2026-05-02T16:31:00+08:00",
          },
        },
        windowLabel: "main",
      });
    });

    await waitFor(() => {
      expect(screen.getByText("v-test-event")).toBeInTheDocument();
      expect(screen.getByTitle("C:\\event\\winget.exe")).toBeInTheDocument();
    });
  });

  it("starts install tasks immediately and updates lock state from install:status", async () => {
    detectAllToolsMock.mockResolvedValue([
      {
        id: "git",
        name: "Git / Git Bash",
        category: "base",
        status: "missing",
        version: undefined,
        executablePath: undefined,
        detectionMethod: "combined",
        lastCheckedAt: "2026-05-02T17:00:00+08:00",
      },
      {
        id: "python",
        name: "Python 3.11",
        category: "base",
        status: "missing",
        version: undefined,
        executablePath: undefined,
        detectionMethod: "combined",
        lastCheckedAt: "2026-05-02T17:00:00+08:00",
      },
    ]);
    installToolMock.mockResolvedValue(undefined);

    render(<App />);

    const installButtons = await screen.findAllByRole("button", { name: installButtonName });
    fireEvent.click(installButtons[0]);

    await waitFor(() => {
      expect(installToolMock).toHaveBeenCalledWith("git");
    });

    await act(async () => {
      installStatusHandler?.({
        event: "install:status",
        id: 2,
        payload: {
          toolId: "git",
          status: "installing",
          phase: "started",
          timestamp: "2026-05-02T17:01:00+08:00",
        },
        windowLabel: "main",
      });
    });

    await waitFor(() => {
      expect(screen.getByRole("button", { name: cancelInstallButtonName })).toBeInTheDocument();
    });

    const disabledInstallButtons = screen.getAllByRole("button", { name: installButtonName });
    expect(disabledInstallButtons[0]).toBeDisabled();
  });

  it("loads install network config and shows non-admin hint", async () => {
    isAdminMock.mockResolvedValue(false);
    getConfigMock.mockResolvedValue({
      installNetwork: {
        mode: "manual_proxy",
        proxyUrl: "http://127.0.0.1:7890",
        npmRegistry: "npmmirror",
      },
      ccswitchDownloadSources: [],
    });
    detectAllToolsMock.mockResolvedValue([]);

    render(<App />);

    await waitFor(() => {
      expect(screen.getByText("manual_proxy")).toBeInTheDocument();
      expect(screen.getByText("npmmirror")).toBeInTheDocument();
      expect(
        screen.getByText("当前不是管理员，安装时可能触发 UAC。"),
      ).toBeInTheDocument();
      expect(
        screen.getByRole("button", { name: "以管理员身份重启" }),
      ).toBeInTheDocument();
    });
  });

  it("releases the install lock after a cancelled status event", async () => {
    detectAllToolsMock.mockResolvedValue([
      {
        id: "git",
        name: "Git / Git Bash",
        category: "base",
        status: "missing",
        version: undefined,
        executablePath: undefined,
        detectionMethod: "combined",
        lastCheckedAt: "2026-05-02T17:10:00+08:00",
      },
      {
        id: "python",
        name: "Python 3.11",
        category: "base",
        status: "missing",
        version: undefined,
        executablePath: undefined,
        detectionMethod: "combined",
        lastCheckedAt: "2026-05-02T17:10:00+08:00",
      },
    ]);
    installToolMock.mockResolvedValue(undefined);
    cancelInstallMock.mockResolvedValue(undefined);

    render(<App />);

    const installButtons = await screen.findAllByRole("button", { name: installButtonName });
    fireEvent.click(installButtons[0]);

    await act(async () => {
      installStatusHandler?.({
        event: "install:status",
        id: 3,
        payload: {
          toolId: "git",
          status: "installing",
          phase: "started",
          timestamp: "2026-05-02T17:11:00+08:00",
        },
        windowLabel: "main",
      });
    });

    fireEvent.click(screen.getByRole("button", { name: cancelInstallButtonName }));

    await waitFor(() => {
      expect(cancelInstallMock).toHaveBeenCalledTimes(1);
    });

    await act(async () => {
      installStatusHandler?.({
        event: "install:status",
        id: 4,
        payload: {
          toolId: "git",
          status: "missing",
          phase: "cancelled",
          timestamp: "2026-05-02T17:11:05+08:00",
        },
        windowLabel: "main",
      });
    });

    await waitFor(() => {
      expect(screen.queryByRole("button", { name: cancelInstallButtonName })).not.toBeInTheDocument();
      const nextInstallButtons = screen.getAllByRole("button", { name: installButtonName });
      expect(nextInstallButtons[0]).not.toBeDisabled();
    });
  });

  it("enables AI install buttons and starts Claude install immediately", async () => {
    detectAllToolsMock.mockResolvedValue([
      {
        id: "claude",
        name: "Claude Code",
        category: "ai",
        status: "missing",
        version: undefined,
        executablePath: undefined,
        detectionMethod: "npm_global_probe",
        lastCheckedAt: "2026-05-03T11:20:00+08:00",
      },
    ]);
    installToolMock.mockResolvedValue(undefined);

    render(<App />);

    const installButton = await screen.findByRole("button", { name: installButtonName });
    fireEvent.click(installButton);

    await waitFor(() => {
      expect(installToolMock).toHaveBeenCalledWith("claude");
    });
  });

  it("saves a manual ccSwitch path and re-detects the tool", async () => {
    detectAllToolsMock.mockResolvedValue([
      {
        id: "ccswitch",
        name: "ccSwitch",
        category: "ai",
        status: "missing",
        version: undefined,
        executablePath: undefined,
        detectionMethod: "path_probe",
        lastCheckedAt: "2026-05-03T11:40:00+08:00",
      },
    ]);
    updateConfigMock.mockResolvedValue({
      installNetwork: {
        mode: "none",
        npmRegistry: "default",
      },
      ccswitchPath: "C:\\Program Files\\ccswitch\\ccswitch.exe",
      ccswitchDownloadSources: [],
    });
    detectToolMock.mockResolvedValue({
      id: "ccswitch",
      name: "ccSwitch",
      category: "ai",
      status: "installed",
      version: undefined,
      executablePath: "C:\\Program Files\\ccswitch\\ccswitch.exe",
      detectionMethod: "path_probe",
      lastCheckedAt: "2026-05-03T11:41:00+08:00",
    });

    render(<App />);

    fireEvent.change(await screen.findByLabelText("ccSwitch executable"), {
      target: { value: "C:\\Program Files\\ccswitch\\ccswitch.exe" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Save ccSwitch path" }));

    await waitFor(() => {
      expect(updateConfigMock).toHaveBeenCalledWith({
        ccswitchPath: "C:\\Program Files\\ccswitch\\ccswitch.exe",
      });
      expect(detectToolMock).toHaveBeenCalledWith("ccswitch");
    });
  });

  it("opens ccSwitch only when a detected executable path exists", async () => {
    detectAllToolsMock.mockResolvedValue([
      {
        id: "ccswitch",
        name: "ccSwitch",
        category: "ai",
        status: "installed",
        version: undefined,
        executablePath: "C:\\Program Files\\ccswitch\\ccswitch.exe",
        detectionMethod: "path_probe",
        lastCheckedAt: "2026-05-03T11:45:00+08:00",
      },
    ]);
    openCcSwitchMock.mockResolvedValue(undefined);

    render(<App />);

    const openButton = await screen.findByRole("button", { name: "Open ccSwitch" });
    expect(openButton).not.toBeDisabled();
    fireEvent.click(openButton);

    await waitFor(() => {
      expect(openCcSwitchMock).toHaveBeenCalledTimes(1);
    });
  });

  it("disables the quick ccSwitch action with the required hint when ccSwitch is not installed", async () => {
    detectAllToolsMock.mockResolvedValue([
      {
        id: "ccswitch",
        name: "ccSwitch",
        category: "ai",
        status: "missing",
        version: undefined,
        executablePath: undefined,
        detectionMethod: "path_probe",
        lastCheckedAt: "2026-05-05T09:00:00+08:00",
      },
    ]);

    render(<App />);

    const openButton = await screen.findByRole("button", { name: "打开 ccSwitch" });
    expect(openButton).toBeDisabled();
    expect(openButton).toHaveAttribute("title", "请先安装或指定 ccSwitch 路径");
  });

  it("saves subscriptionPageUrl and opens the configured subscription page from quick actions", async () => {
    detectAllToolsMock.mockResolvedValue([]);
    getConfigMock.mockResolvedValue({
      installNetwork: {
        mode: "none",
        npmRegistry: "default",
      },
      ccswitchPath: undefined,
      ccswitchDownloadSources: [],
      subscriptionPageUrl: "https://old.example.com/subscription",
    });
    updateConfigMock.mockResolvedValue({
      installNetwork: {
        mode: "none",
        npmRegistry: "default",
      },
      ccswitchPath: undefined,
      ccswitchDownloadSources: [],
      subscriptionPageUrl: "https://nodes.example.com/dashboard",
    });
    openSubscriptionPageMock.mockResolvedValue(undefined);

    render(<App />);

    fireEvent.change(await screen.findByLabelText("节点订阅网页"), {
      target: { value: "https://nodes.example.com/dashboard" },
    });
    fireEvent.click(screen.getByRole("button", { name: "保存订阅网页" }));

    await waitFor(() => {
      expect(updateConfigMock).toHaveBeenCalledWith({
        subscriptionPageUrl: "https://nodes.example.com/dashboard",
      });
    });

    await waitFor(() => {
      expect(screen.getByRole("button", { name: "打开节点订阅网页" })).not.toBeDisabled();
    });
    fireEvent.click(screen.getByRole("button", { name: "打开节点订阅网页" }));

    await waitFor(() => {
      expect(openSubscriptionPageMock).toHaveBeenCalledTimes(1);
    });
  });

  it("quick actions re-detect all and jump to settings and logs areas", async () => {
    detectAllToolsMock.mockResolvedValue([]);
    const scrollIntoViewMock = vi.fn();
    const originalGetElementById = document.getElementById.bind(document);
    vi.spyOn(document, "getElementById").mockImplementation((id: string) => {
      const element = originalGetElementById(id);
      if (element) {
        Object.defineProperty(element, "scrollIntoView", {
          configurable: true,
          value: scrollIntoViewMock,
        });
      }
      return element;
    });

    render(<App />);

    await waitFor(() => {
      expect(detectAllToolsMock).toHaveBeenCalledTimes(1);
    });

    const quickActions = screen.getByText("Quick Actions").closest("section") as HTMLElement;
    fireEvent.click(within(quickActions).getByRole("button", { name: "重新检测全部" }));
    fireEvent.click(within(quickActions).getByRole("button", { name: "打开安装网络设置" }));
    fireEvent.click(within(quickActions).getByRole("button", { name: "查看日志" }));

    await waitFor(() => {
      expect(detectAllToolsMock).toHaveBeenCalledTimes(2);
      expect(document.getElementById).toHaveBeenCalledWith("install-network-settings");
      expect(document.getElementById).toHaveBeenCalledWith("logs-panel");
      expect(scrollIntoViewMock).toHaveBeenCalled();
    });
  });

  it("shows the configured ccSwitch download source count", async () => {
    getConfigMock.mockResolvedValue({
      installNetwork: {
        mode: "none",
        npmRegistry: "default",
      },
      ccswitchPath: undefined,
      ccswitchDownloadSources: [
        {
          name: "Official TODO",
          url: "https://example.invalid/ccswitch.exe",
          priority: 10,
          enabled: true,
          kind: "direct_exe",
        },
      ],
    });
    detectAllToolsMock.mockResolvedValue([]);

    render(<App />);

    await waitFor(() => {
      expect(screen.getByText(/Configured download sources: 1/)).toBeInTheDocument();
    });
  });

  it("starts reinstall and latest install actions immediately for npm AI tools", async () => {
    detectAllToolsMock.mockResolvedValue([
      {
        id: "codex",
        name: "Codex CLI",
        category: "ai",
        status: "installed",
        version: "0.1.0",
        executablePath: "C:\\Users\\me\\AppData\\Roaming\\npm\\codex.cmd",
        detectionMethod: "npm_global_probe",
        lastCheckedAt: "2026-05-03T13:05:00+08:00",
      },
    ]);
    reinstallToolMock.mockResolvedValue(undefined);
    installLatestToolMock.mockResolvedValue(undefined);

    render(<App />);

    fireEvent.click(await screen.findByRole("button", { name: "重试安装" }));
    fireEvent.click(screen.getByRole("button", { name: "安装最新版" }));

    await waitFor(() => {
      expect(reinstallToolMock).toHaveBeenCalledWith("codex");
      expect(installLatestToolMock).toHaveBeenCalledWith("codex");
    });
  });

  it("shows PATH repair instructions only for installed tools missing PATH and copies a manual user PATH command", async () => {
    detectAllToolsMock.mockResolvedValue([
      {
        id: "codex",
        name: "Codex CLI",
        category: "ai",
        status: "installed_but_path_missing",
        version: "0.1.0",
        executablePath: "C:\\Users\\me\\AppData\\Roaming\\npm\\codex.cmd",
        detectionMethod: "npm_global_probe",
        lastCheckedAt: "2026-05-04T22:30:00+08:00",
      },
      {
        id: "git",
        name: "Git / Git Bash",
        category: "base",
        status: "installed",
        version: "2.45.0",
        executablePath: "C:\\Program Files\\Git\\cmd\\git.exe",
        detectionMethod: "combined",
        lastCheckedAt: "2026-05-04T22:30:00+08:00",
      },
    ]);

    render(<App />);

    const repairButton = await screen.findByRole("button", {
      name: "PATH repair instructions",
    });
    expect(screen.getAllByRole("button", { name: "PATH repair instructions" })).toHaveLength(1);

    fireEvent.click(repairButton);

    const dialog = await screen.findByRole("dialog", {
      name: "PATH repair instructions",
    });
    const modal = within(dialog);
    expect(dialog).toBeInTheDocument();
    expect(modal.getByText("Codex CLI")).toBeInTheDocument();
    expect(modal.getByText("C:\\Users\\me\\AppData\\Roaming\\npm\\codex.cmd")).toBeInTheDocument();
    expect(modal.getByText("C:\\Users\\me\\AppData\\Roaming\\npm")).toBeInTheDocument();
    expect(
      modal.getByText(/This client will not automatically modify PATH/i),
    ).toBeInTheDocument();

    fireEvent.click(modal.getByRole("button", { name: "Copy manual PATH command" }));

    await waitFor(() => {
      expect(clipboardWriteTextMock).toHaveBeenCalledTimes(1);
    });
    const copiedCommand = clipboardWriteTextMock.mock.calls[0][0];
    expect(copiedCommand).toContain("[Environment]::SetEnvironmentVariable");
    expect(copiedCommand).toContain('"User"');
    expect(copiedCommand).toContain("C:\\Users\\me\\AppData\\Roaming\\npm");
    expect(copiedCommand).toContain("-notcontains");
    expect(copiedCommand).not.toContain("setx");
  });

  it("keeps the log viewer to the latest 5000 lines and exposes diagnostics actions", async () => {
    detectAllToolsMock.mockResolvedValue([]);
    getLogPreviewMock.mockResolvedValue(["existing log line"]);

    render(<App />);

    await waitFor(() => {
      expect(getLogPreviewMock).toHaveBeenCalledWith(5000);
      expect(screen.getByText("existing log line")).toBeInTheDocument();
    });

    await act(async () => {
      for (let index = 0; index < 5002; index += 1) {
        installProgressHandler?.({
          event: "install:progress",
          id: index,
          payload: {
            toolId: "codex",
            phase: "running",
            stream: "stdout",
            line: `progress line ${index}`,
            timestamp: `2026-05-05T10:00:${String(index % 60).padStart(2, "0")}+08:00`,
          },
          windowLabel: "main",
        });
      }
    });

    await waitFor(() => {
      expect(screen.queryByText("existing log line")).not.toBeInTheDocument();
      expect(screen.queryByText("progress line 1")).not.toBeInTheDocument();
      expect(screen.getByText(/progress line 2$/)).toBeInTheDocument();
      expect(screen.getByText(/progress line 5001$/)).toBeInTheDocument();
    });

    fireEvent.click(screen.getByRole("button", { name: "Open full log file" }));
    fireEvent.click(screen.getByRole("button", { name: "Open log directory" }));
    fireEvent.click(screen.getByRole("button", { name: "Export diagnostics zip" }));

    await waitFor(() => {
      expect(openFullLogFileMock).toHaveBeenCalledTimes(1);
      expect(openLogDirectoryMock).toHaveBeenCalledTimes(1);
      expect(exportDiagnosticsLogZipMock).toHaveBeenCalledTimes(1);
      expect(screen.getByText("Diagnostics zip exported: C:\\logs\\diagnostics.zip")).toBeInTheDocument();
    });
  });
});
