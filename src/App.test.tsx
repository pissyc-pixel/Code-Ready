/// <reference types="vitest" />

import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import type { Event } from "@tauri-apps/api/event";
import type { AppConfig } from "./types/config";
import type { DetectResultEvent, InstallStatusEvent } from "./types/events";
import type { ToolStatus } from "./types/tool";

const detectAllToolsMock = vi.fn<() => Promise<ToolStatus[]>>();
const detectToolMock = vi.fn();
const installToolMock = vi.fn<() => Promise<void>>();
const cancelInstallMock = vi.fn<() => Promise<void>>();
const getConfigMock = vi.fn<() => Promise<AppConfig>>();
const updateConfigMock = vi.fn();
const resetConfigMock = vi.fn();
const isAdminMock = vi.fn<() => Promise<boolean>>();
const openCcSwitchMock = vi.fn<() => Promise<void>>();
const restartAsAdminMock = vi.fn<() => Promise<void>>();
const listenMock = vi.fn();

vi.mock("./lib/api", () => ({
  detectAllTools: () => detectAllToolsMock(),
  detectTool: (...args: unknown[]) => detectToolMock(...args),
  installTool: (...args: unknown[]) => installToolMock(...args),
  cancelInstall: (...args: unknown[]) => cancelInstallMock(...args),
  getConfig: () => getConfigMock(),
  updateConfig: (...args: unknown[]) => updateConfigMock(...args),
  resetConfig: () => resetConfigMock(),
  isAdmin: () => isAdminMock(),
  openCcSwitch: () => openCcSwitchMock(),
  restartAsAdmin: () => restartAsAdminMock(),
}));

let detectResultHandler:
  | ((event: Event<DetectResultEvent>) => void)
  | null = null;
let installStatusHandler:
  | ((event: Event<InstallStatusEvent>) => void)
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
    }
    return Promise.resolve(() => Promise.resolve());
  },
}));

import App from "./App";

describe("App", () => {
  beforeEach(() => {
    installProgressSubscribed = false;
    getConfigMock.mockResolvedValue({
      installNetwork: {
        mode: "none",
        npmRegistry: "default",
      },
      ccswitchPath: undefined,
      ccswitchDownloadSources: [],
    });
    isAdminMock.mockResolvedValue(true);
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

    const installButtons = await screen.findAllByRole("button", { name: "安装" });
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
      expect(screen.getByRole("button", { name: "取消安装" })).toBeInTheDocument();
    });

    const disabledInstallButtons = screen.getAllByRole("button", { name: "安装" });
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

    const installButtons = await screen.findAllByRole("button", { name: "安装" });
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

    fireEvent.click(screen.getByRole("button", { name: "取消安装" }));

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
      expect(screen.queryByRole("button", { name: "取消安装" })).not.toBeInTheDocument();
      const nextInstallButtons = screen.getAllByRole("button", { name: "安装" });
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

    const installButton = await screen.findByRole("button", { name: "安装" });
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
});
