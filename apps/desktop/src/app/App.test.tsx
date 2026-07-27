import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, expect, test, vi } from "vitest";

import type {
  AppSnapshot,
  DetectionRun,
  ToolObservation,
} from "../shared/api/generated";
import type {
  DetectionSnapshotState,
} from "../features/detection/useDetectionSnapshot";
import { useDetectionSnapshot } from "../features/detection/useDetectionSnapshot";
import App from "./App";

vi.mock("../features/detection/useDetectionSnapshot", () => ({
  useDetectionSnapshot: vi.fn(),
}));

const mockedUseDetectionSnapshot = vi.mocked(useDetectionSnapshot);

const git: ToolObservation = {
  toolId: "git",
  state: "presentHealthy",
  version: "2.47.1",
  versionStatus: "notComparable",
  evidence: {
    code: "pathCommandHealthy",
    displayPath: "~/bin/git",
    exit: "success",
  },
  checkedAtEpochMs: 1_754_000_000_000,
};

const completedRun: DetectionRun = {
  id: "run-1",
  requestedToolIds: ["git"],
  status: "completed",
  startedAtEpochMs: 1_754_000_000_000,
  finishedAtEpochMs: 1_754_000_000_100,
  errorCode: null,
};

const appSnapshot: AppSnapshot = {
  schemaVersion: 2,
  snapshotVersion: 1,
  lastEventSequence: 0,
  platform: "macosArm64",
  tools: [
    {
      id: "git",
      labelKey: "tools.git.name",
      platformPolicies: [],
      capabilities: ["detect"],
      runtimeDependencies: [],
    },
    {
      id: "claudeCode",
      labelKey: "tools.claudeCode.name",
      platformPolicies: [],
      capabilities: ["detect"],
      runtimeDependencies: [],
    },
    {
      id: "codexCli",
      labelKey: "tools.codexCli.name",
      platformPolicies: [],
      capabilities: ["detect"],
      runtimeDependencies: [],
    },
  ],
  observations: [],
  detectionRun: null,
};

function state(
  overrides: Partial<DetectionSnapshotState> = {},
): DetectionSnapshotState {
  return {
    phase: "ready",
    snapshot: appSnapshot,
    syncWarning: null,
    detectionStartError: null,
    refresh: vi.fn().mockResolvedValue(undefined),
    startDetection: vi.fn().mockResolvedValue(undefined),
    ...overrides,
  };
}

afterEach(() => {
  cleanup();
});

beforeEach(() => {
  mockedUseDetectionSnapshot.mockReset();
});

test("shows loading while the first snapshot is unavailable", () => {
  mockedUseDetectionSnapshot.mockReturnValue(state({ phase: "loading", snapshot: null }));

  render(<App />);

  expect(screen.getByText("正在读取当前设备…")).toBeInTheDocument();
});

test("shows a retryable bootstrap error", () => {
  const refresh = vi.fn().mockResolvedValue(undefined);
  mockedUseDetectionSnapshot.mockReturnValue(
    state({ phase: "bootstrapError", snapshot: null, refresh }),
  );

  render(<App />);

  expect(screen.getByText("暂时无法读取设备状态。")).toBeInTheDocument();
  fireEvent.click(screen.getByRole("button", { name: "重试" }));
  expect(refresh).toHaveBeenCalledOnce();
});

test("shows onboarding for the first process-local run", () => {
  mockedUseDetectionSnapshot.mockReturnValue(state());

  render(<App />);

  expect(screen.getByRole("heading", { name: "先看看这台电脑的开发环境" })).toBeInTheDocument();
});

test("reload with a completed run opens the status center", () => {
  mockedUseDetectionSnapshot.mockReturnValue(
    state({ snapshot: { ...appSnapshot, observations: [git], detectionRun: completedRun } }),
  );

  render(<App />);

  expect(screen.getByRole("heading", { name: "开发环境状态" })).toBeInTheDocument();
});

test("completing onboarding enters the status center without persistence", () => {
  let current = state();
  mockedUseDetectionSnapshot.mockImplementation(() => current);
  const view = render(<App />);

  expect(screen.getByRole("button", { name: "开始检测" })).toBeInTheDocument();
  current = state({
    snapshot: { ...appSnapshot, observations: [git], detectionRun: completedRun },
  });
  view.rerender(<App />);

  fireEvent.click(screen.getByRole("button", { name: "进入状态中心" }));
  expect(screen.getByRole("heading", { name: "开发环境状态" })).toBeInTheDocument();
});

test("shows a sync warning without blocking the status center", () => {
  mockedUseDetectionSnapshot.mockReturnValue(state({
    snapshot: { ...appSnapshot, observations: [git], detectionRun: completedRun },
    syncWarning: "refreshFailed",
  }));

  render(<App />);

  expect(screen.getByRole("heading", { name: "开发环境状态" })).toBeInTheDocument();
  expect(screen.getByText("暂时无法刷新，下面仍显示最近一次已知状态。")).toBeInTheDocument();
});
