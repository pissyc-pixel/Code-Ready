import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, expect, test, vi } from "vitest";

import type {
  AppSnapshot,
  DetectionRun,
  ToolDefinition,
  ToolObservation,
} from "../../shared/api/generated";
import StatusCenter from "./StatusCenter";

afterEach(() => {
  cleanup();
});

const tools: ToolDefinition[] = [
  {
    id: "winget",
    labelKey: "tools.winget.name",
    platformPolicies: [],
    capabilities: ["detect", "install", "upgrade", "repair"],
    runtimeDependencies: [],
  },
  {
    id: "git",
    labelKey: "tools.git.name",
    platformPolicies: [],
    capabilities: ["detect"],
    runtimeDependencies: [],
  },
  {
    id: "nodejs",
    labelKey: "tools.nodejs.name",
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
];

function observation(
  overrides: Partial<ToolObservation> = {},
): ToolObservation {
  return {
    toolId: "git",
    state: "presentHealthy",
    version: "2.47.1",
    versionStatus: "notComparable",
    evidence: {
      code: "pathCommandHealthy",
      displayPath: "~/bin/git",
      exit: "success",
    },
    checkedAtEpochMs: 1_754_000_000_000n,
    ...overrides,
  };
}

function runningRun(): DetectionRun {
  return {
    id: "run-1",
    requestedToolIds: ["git", "claudeCode", "codexCli"],
    status: "running",
    startedAtEpochMs: 1_754_000_000_000n,
    finishedAtEpochMs: null,
    errorCode: null,
  };
}

function snapshot(
  platform: AppSnapshot["platform"] = "macosArm64",
  observations: ToolObservation[] = [],
  detectionRun: DetectionRun | null = null,
): AppSnapshot {
  return {
    schemaVersion: 2,
    snapshotVersion: 1n,
    lastEventSequence: 0n,
    platform,
    tools,
    observations,
    detectionRun,
  };
}

function renderStatus(
  nextSnapshot: AppSnapshot = snapshot("macosArm64", [observation()]),
  overrides: Partial<React.ComponentProps<typeof StatusCenter>> = {},
) {
  return render(
    <StatusCenter
      snapshot={nextSnapshot}
      startDetection={vi.fn()}
      refresh={vi.fn()}
      syncWarning={null}
      detectionStartError={null}
      {...overrides}
    />,
  );
}

test.each([
  ["absent", "未检测到"],
  ["presentHealthy", "可以正常运行"],
  ["presentPathIssue", "已安装，但当前环境路径没有指向它"],
  ["presentBroken", "已找到，但无法正常完成版本检查"],
  ["unknown", "暂时无法判断"],
] as const)("renders %s as a fact, not a task state", (state, label) => {
  renderStatus(snapshot("macosArm64", [observation({ state })]));

  expect(screen.getByText(label)).toBeInTheDocument();
  expect(screen.queryByText(/downloading|executing|cancelled/)).not.toBeInTheDocument();
});

test("shows parsed version and notComparable independently", () => {
  renderStatus(snapshot("macosArm64", [
    observation({
      state: "presentHealthy",
      version: "0.138.0",
      versionStatus: "notComparable",
    }),
  ]));

  expect(screen.getByText("版本 0.138.0")).toBeInTheDocument();
  expect(screen.getByText("已检测到版本，但本版本暂不判断新旧")).toBeInTheDocument();
  expect(screen.queryByText(/最新|需要升级|过旧/)).not.toBeInTheDocument();
});

test("starts one-tool and full read-only detection", () => {
  const startDetection = vi.fn();
  renderStatus(snapshot(), { startDetection });

  fireEvent.click(screen.getByRole("button", { name: "重新检测 Git" }));
  expect(startDetection).toHaveBeenCalledWith(["git"]);

  fireEvent.click(screen.getByRole("button", { name: "重新检测全部" }));
  expect(startDetection).toHaveBeenCalledWith(undefined);
});

test.each([
  ["windowsX64", "Windows 11 x64"],
  ["macosArm64", "macOS Apple Silicon"],
] as const)("keeps identical controls and state semantics on %s", (platform, label) => {
  renderStatus(snapshot(platform));

  expect(screen.getByText(label)).toBeInTheDocument();
  expect(screen.getAllByRole("button").map((button) => button.textContent)).toEqual([
    "刷新状态",
    "重新检测全部",
    "重新检测 Git",
    "重新检测 Claude Code",
    "重新检测 Codex CLI",
  ]);
});

test("keeps old facts visible while a run is active and leaves refresh enabled", () => {
  renderStatus(snapshot("macosArm64", [observation()], runningRun()));

  expect(screen.getByRole("status")).toHaveTextContent("检测正在进行");
  expect(screen.getByText("可以正常运行")).toBeInTheDocument();
  expect(screen.getByRole("button", { name: "刷新状态" })).not.toBeDisabled();
  expect(screen.getByRole("button", { name: "重新检测全部" })).toBeDisabled();
});

test("renders only Slice 1 tools and presentation-only empty states", () => {
  renderStatus(snapshot());

  expect(screen.getByText("Git")).toBeInTheDocument();
  expect(screen.getByText("Claude Code")).toBeInTheDocument();
  expect(screen.getByText("Codex CLI")).toBeInTheDocument();
  expect(screen.queryByText("Winget")).not.toBeInTheDocument();
  expect(screen.queryByText("Node.js")).not.toBeInTheDocument();
  expect(screen.getAllByText("尚未检测")).toHaveLength(3);
});
