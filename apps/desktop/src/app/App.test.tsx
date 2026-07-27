import { render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, test, vi } from "vitest";

import type { AppSnapshot } from "../shared/api/generated";
import { bootstrap, subscribeDetectionChanged } from "../shared/api/client";
import App from "./App";

vi.mock("../shared/api/client", () => ({
  bootstrap: vi.fn(),
  detectTools: vi.fn(),
  subscribeDetectionChanged: vi.fn(),
  commandErrorCode: vi.fn(),
}));

const mockedBootstrap = vi.mocked(bootstrap);
const mockedSubscribe = vi.mocked(subscribeDetectionChanged);

const appSnapshot: AppSnapshot = {
  schemaVersion: 2,
  snapshotVersion: 1n,
  lastEventSequence: 0n,
  platform: "macosArm64",
  tools: [
    {
      id: "git",
      labelKey: "tools.git.name",
      platformPolicies: [
        { platform: "windowsX64", requirement: "default" },
        { platform: "macosArm64", requirement: "default" },
      ],
      capabilities: ["detect", "install", "upgrade", "repair"],
      runtimeDependencies: [],
    },
  ],
  observations: [],
  detectionRun: null,
};

describe("App", () => {
  beforeEach(() => {
    mockedBootstrap.mockReset();
    mockedSubscribe.mockReset();
    mockedSubscribe.mockResolvedValue(() => undefined);
  });

  test("shows the loading state before bootstrap is available", () => {
    mockedBootstrap.mockReturnValue(new Promise(() => undefined));

    render(<App />);

    expect(screen.getByText("正在读取当前设备…")).toBeInTheDocument();
  });

  test("shows the current platform and registry summary without action controls", async () => {
    mockedBootstrap.mockResolvedValue(appSnapshot);

    render(<App />);

    expect(await screen.findByText("Code-Ready V2")).toBeInTheDocument();
    expect(screen.getByText("macOS Apple Silicon")).toBeInTheDocument();
    expect(screen.getByText("已载入 1 项内置工具定义")).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: /安装|提权|登录|API Key|遥测/ })).not.toBeInTheDocument();
  });
});
