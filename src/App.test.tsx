/// <reference types="vitest" />

import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
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
const selectCcSwitchExecutableMock = vi.fn<() => Promise<string | null>>();
const listenMock = vi.fn();

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
  selectCcSwitchExecutable: () => selectCcSwitchExecutableMock(),
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

describe("App desktop experience", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    detectResultHandler = null;
    installStatusHandler = null;
    installProgressHandler = null;
    getConfigMock.mockResolvedValue({
      installNetwork: {
        mode: "none",
        npmRegistry: "default",
      },
      ccswitchPath: undefined,
      ccswitchDownloadSources: [
        {
          name: "mirror-a",
          url: "https://a",
          priority: 1,
          enabled: true,
          kind: "direct_exe",
        },
      ],
      subscriptionPageUrl: "https://nodes.example.com/dashboard",
    });
    updateConfigMock.mockImplementation(async (patch: Partial<AppConfig>) => ({
      installNetwork: {
        mode: "none",
        npmRegistry: "default",
        ...(patch.installNetwork ?? {}),
      },
      ccswitchPath: patch.ccswitchPath,
      ccswitchDownloadSources: [
        {
          name: "mirror-a",
          url: "https://a",
          priority: 1,
          enabled: true,
          kind: "direct_exe",
        },
      ],
      subscriptionPageUrl: patch.subscriptionPageUrl ?? "https://nodes.example.com/dashboard",
    }));
    isAdminMock.mockResolvedValue(true);
    getLogPreviewMock.mockResolvedValue(["existing log line"]);
    openCcSwitchMock.mockResolvedValue(undefined);
    openSubscriptionPageMock.mockResolvedValue(undefined);
    restartAsAdminMock.mockResolvedValue(undefined);
    openFullLogFileMock.mockResolvedValue(undefined);
    openLogDirectoryMock.mockResolvedValue(undefined);
    exportDiagnosticsLogZipMock.mockResolvedValue({ path: "C:\\logs\\diagnostics.zip" });
    selectCcSwitchExecutableMock.mockResolvedValue(null);
  });

  afterEach(() => {
    cleanup();
  });

  it("renders the Code-ready shell and dashboard context", async () => {
    detectAllToolsMock.mockResolvedValue([
      makeTool({ id: "git", name: "Git / Git Bash", category: "base" }),
      makeTool({ id: "node", name: "Node.js", category: "base" }),
      makeTool({ id: "codex", name: "Codex CLI", category: "ai" }),
      makeTool({
        id: "ccswitch",
        name: "ccSwitch",
        category: "ai",
        executablePath: "D:\\ccSwitch\\cc-switch.exe",
      }),
    ]);

    render(<App />);

    await waitFor(() => {
      expect(detectAllToolsMock).toHaveBeenCalledTimes(1);
      expect(getConfigMock).toHaveBeenCalledTimes(1);
      expect(isAdminMock).toHaveBeenCalledTimes(1);
    });

    expect(screen.getByRole("heading", { name: "Code-ready" })).toBeInTheDocument();
    expect(screen.getByText("AI 编码环境准备工具")).toBeInTheDocument();
    expect(screen.getByText("existing log line")).toBeInTheDocument();
    expect(document.title).toBe("Code-ready");
  });

  it("removes login/config prompts on the ai tools page and shows ccswitch version fallback", async () => {
    detectAllToolsMock.mockResolvedValue([
      makeTool({
        id: "claude",
        name: "Claude Code",
        category: "ai",
        suggestion:
          "Detected CLI files, but login or local configuration may still be required.",
      }),
      makeTool({
        id: "ccswitch",
        name: "ccSwitch",
        category: "ai",
        version: undefined,
        executablePath: "D:\\ccSwitch\\cc-switch.exe",
      }),
    ]);

    render(<App />);

    fireEvent.click(await screen.findByRole("button", { name: /^AI 工具/ }));
    await screen.findByRole("region", { name: "AI 工具" });

    expect(
      screen.queryByText(
        "Detected CLI files, but login or local configuration may still be required.",
      ),
    ).not.toBeInTheDocument();
    expect(screen.getByText("版本未提供")).toBeInTheDocument();
  });

  it.skip("supports browsing and saving for ccswitch", async () => {
    detectAllToolsMock.mockResolvedValue([
      makeTool({
        id: "ccswitch",
        name: "ccSwitch",
        category: "ai",
        version: undefined,
        executablePath: "D:\\ccSwitch\\cc-switch.exe",
      }),
    ]);
    selectCcSwitchExecutableMock.mockResolvedValue("D:\\ccSwitch\\cc-switch.exe");

    render(<App />);

    fireEvent.click(await screen.findByRole("button", { name: /^ccSwitch/ }));
    await screen.findByRole("region", { name: "ccSwitch" });

    fireEvent.click(screen.getByRole("button", { name: "浏览..." }));
    await waitFor(() => {
      expect(selectCcSwitchExecutableMock).toHaveBeenCalledTimes(1);
    });
    expect(screen.getByDisplayValue("D:\\ccSwitch\\cc-switch.exe")).toBeInTheDocument();

    fireEvent.click(screen.getAllByRole("button", { name: "保存" })[0]);
    await waitFor(() => {
      expect(updateConfigMock).toHaveBeenCalledWith({
        ccswitchPath: "D:\\ccSwitch\\cc-switch.exe",
      });
    });
    expect(detectToolMock).not.toHaveBeenCalled();

    expect(screen.getAllByText("版本未提供").length).toBeGreaterThan(0);
  });

  it.skip("keeps ccswitch open and subscription actions wired", async () => {
    detectAllToolsMock.mockResolvedValue([
      makeTool({
        id: "ccswitch",
        name: "ccSwitch",
        category: "ai",
        executablePath: "D:\\ccSwitch\\cc-switch.exe",
      }),
    ]);

    render(<App />);

    fireEvent.click(await screen.findByRole("button", { name: /^ccSwitch/ }));
    await screen.findByRole("region", { name: "ccSwitch" });

    fireEvent.click(screen.getByRole("button", { name: "打开 ccSwitch" }));
    fireEvent.click(screen.getByRole("button", { name: "打开" }));

    await waitFor(() => {
      expect(openCcSwitchMock).toHaveBeenCalledTimes(1);
      expect(openSubscriptionPageMock).toHaveBeenCalledTimes(1);
    });
  });
});
