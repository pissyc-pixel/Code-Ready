/// <reference types="vitest" />

import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import type { Event } from "@tauri-apps/api/event";
import type { DetectResultEvent, InstallStatusEvent } from "./types/events";
import type { ToolStatus } from "./types/tool";

const detectAllToolsMock = vi.fn<() => Promise<ToolStatus[]>>();
const detectToolMock = vi.fn();
const installToolMock = vi.fn<() => Promise<void>>();
const cancelInstallMock = vi.fn<() => Promise<void>>();
const listenMock = vi.fn();

vi.mock("./lib/api", () => ({
  detectAllTools: () => detectAllToolsMock(),
  detectTool: (...args: unknown[]) => detectToolMock(...args),
  installTool: (...args: unknown[]) => installToolMock(...args),
  cancelInstall: (...args: unknown[]) => cancelInstallMock(...args),
}));

let detectResultHandler:
  | ((event: Event<DetectResultEvent>) => void)
  | null = null;
let installStatusHandler:
  | ((event: Event<InstallStatusEvent>) => void)
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
    return Promise.resolve(() => Promise.resolve());
  },
}));

import App from "./App";

describe("App", () => {
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
});
