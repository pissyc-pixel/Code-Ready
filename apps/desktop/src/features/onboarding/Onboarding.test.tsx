import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, expect, test, vi } from "vitest";

import type {
  AppSnapshot,
  DetectionRun,
  ToolObservation,
} from "../../shared/api/generated";
import Onboarding from "./Onboarding";

afterEach(() => {
  cleanup();
});

function snapshot(
  detectionRun: DetectionRun | null,
  observations: ToolObservation[],
): AppSnapshot {
  return {
    schemaVersion: 2,
    snapshotVersion: 1n,
    lastEventSequence: 0n,
    platform: "macosArm64",
    tools: [],
    observations,
    detectionRun,
  };
}

function run(status: DetectionRun["status"]): DetectionRun {
  return {
    id: "run-1",
    requestedToolIds: ["git", "claudeCode", "codexCli"],
    status,
    startedAtEpochMs: 1_754_000_000_000n,
    finishedAtEpochMs: status === "running" ? null : 1_754_000_000_100n,
    errorCode: status === "failed" ? "internal" : null,
  };
}

function fact(toolId: ToolObservation["toolId"]): ToolObservation {
  return {
    toolId,
    state: "presentHealthy",
    version: toolId === "git" ? "2.47.1" : "0.138.0",
    versionStatus: "notComparable",
    evidence: {
      code: "pathCommandHealthy",
      displayPath: "/fake/tool",
      exit: "success",
    },
    checkedAtEpochMs: 1_754_000_000_000n,
  };
}

test("runs the approved process-local onboarding flow", async () => {
  const startDetection = vi.fn().mockResolvedValue(undefined);
  const onComplete = vi.fn();
  const { rerender } = render(
    <Onboarding
      snapshot={snapshot(null, [])}
      startDetection={startDetection}
      onComplete={onComplete}
    />,
  );

  expect(
    screen.getByRole("heading", { name: "先看看这台电脑的开发环境" }),
  ).toBeInTheDocument();
  expect(screen.getByText(/不会安装、升级或修复/)).toBeInTheDocument();
  expect(screen.getByText(/不上传遥测/)).toBeInTheDocument();

  fireEvent.click(screen.getByRole("button", { name: "开始检测" }));
  expect(startDetection).toHaveBeenCalledWith(undefined);

  rerender(
    <Onboarding
      snapshot={snapshot(run("running"), [])}
      startDetection={startDetection}
      onComplete={onComplete}
    />,
  );
  expect(screen.getByRole("status")).toHaveTextContent("检测正在进行");

  rerender(
    <Onboarding
      snapshot={snapshot(run("completed"), [
        fact("git"),
        fact("claudeCode"),
        fact("codexCli"),
      ])}
      startDetection={startDetection}
      onComplete={onComplete}
    />,
  );
  fireEvent.click(screen.getByRole("button", { name: "进入状态中心" }));
  expect(onComplete).toHaveBeenCalledOnce();
});

test("keeps failed-run facts, exposes semantic errors, and has no privileged actions", () => {
  const startDetection = vi.fn().mockResolvedValue(undefined);
  render(
    <Onboarding
      snapshot={snapshot(run("failed"), [fact("git")])}
      startDetection={startDetection}
      onComplete={vi.fn()}
      detectionStartError="internal"
    />,
  );

  expect(screen.getByText("检测任务意外停止，工具卡片仍显示最近一次已知事实。")).toBeInTheDocument();
  expect(screen.getByText("暂时无法开始检测，请重试。")).toBeInTheDocument();
  expect(screen.getByText("可以正常运行")).toBeInTheDocument();
  expect(screen.getAllByRole("heading")).toHaveLength(2);
  expect(screen.getAllByRole("heading")[0].tagName).toBe("H1");
  expect(screen.getAllByRole("heading")[1].tagName).toBe("H2");
  expect(screen.getAllByRole("button").every((button) => button.textContent !== "")).toBe(true);
  expect(
    screen.queryByRole("button", { name: /安装|升级|修复|管理员|登录|API Key|代理|日志/ }),
  ).not.toBeInTheDocument();

  fireEvent.click(screen.getAllByRole("button", { name: "重试" })[0]);
  expect(startDetection).toHaveBeenCalledWith(undefined);
});

test("renders fact and notComparable version messages independently", () => {
  render(
    <Onboarding
      snapshot={snapshot(run("completed"), [fact("git")])}
      startDetection={vi.fn()}
      onComplete={vi.fn()}
    />,
  );

  expect(screen.getByText("可以正常运行")).toBeInTheDocument();
  expect(screen.getByText("版本 2.47.1")).toBeInTheDocument();
  expect(screen.getByText("已检测到版本，但本版本暂不判断新旧")).toBeInTheDocument();
  expect(screen.queryByText(/最新|需要升级|过旧/)).not.toBeInTheDocument();
});
