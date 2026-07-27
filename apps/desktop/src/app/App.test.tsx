import { render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, test, vi } from "vitest";

import type { BootstrapState } from "../shared/api/generated";
import { getBootstrapState } from "../shared/api/client";
import App from "./App";

vi.mock("../shared/api/client", () => ({
  getBootstrapState: vi.fn(),
}));

const mockedGetBootstrapState = vi.mocked(getBootstrapState);

const bootstrapState: BootstrapState = {
  schemaVersion: 1,
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
};

describe("App", () => {
  beforeEach(() => {
    mockedGetBootstrapState.mockReset();
  });

  test("shows the loading state before bootstrap is available", () => {
    mockedGetBootstrapState.mockReturnValue(new Promise(() => undefined));

    render(<App />);

    expect(screen.getByText("正在读取当前设备…")).toBeInTheDocument();
  });

  test("shows the current platform and registry summary without action controls", async () => {
    mockedGetBootstrapState.mockResolvedValue(bootstrapState);

    render(<App />);

    expect(await screen.findByText("Code-Ready V2")).toBeInTheDocument();
    expect(screen.getByText("macOS Apple Silicon")).toBeInTheDocument();
    expect(screen.getByText("已载入 1 项内置工具定义")).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: /安装|提权|登录|API Key|遥测/ })).not.toBeInTheDocument();
  });
});
