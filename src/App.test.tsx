/// <reference types="vitest" />

import { act, render, screen, waitFor } from "@testing-library/react";
import type { Event } from "@tauri-apps/api/event";
import type { DetectResultEvent } from "./types/events";
import type { ToolStatus } from "./types/tool";

const detectAllToolsMock = vi.fn<() => Promise<ToolStatus[]>>();
const detectToolMock = vi.fn();
const listenMock = vi.fn();

vi.mock("./lib/api", () => ({
  detectAllTools: () => detectAllToolsMock(),
  detectTool: (...args: unknown[]) => detectToolMock(...args),
}));

let detectResultHandler:
  | ((event: Event<DetectResultEvent>) => void)
  | null = null;

vi.mock("@tauri-apps/api/event", () => ({
  listen: (eventName: string, handler: (event: Event<DetectResultEvent>) => void) => {
    listenMock(eventName);
    detectResultHandler = handler;
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
});
