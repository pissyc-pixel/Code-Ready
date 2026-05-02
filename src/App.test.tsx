/// <reference types="vitest" />

import { render, screen } from "@testing-library/react";
import App from "./App";

describe("App", () => {
  it("renders the V0.1 mock dashboard sections and representative tool actions", () => {
    render(<App />);

    expect(
      screen.getByRole("heading", { name: "AI Coding 环境助手" }),
    ).toBeInTheDocument();
    expect(screen.getByText("基础环境")).toBeInTheDocument();
    expect(screen.getByText("AI Coding 工具")).toBeInTheDocument();
    expect(screen.getByText("安装网络")).toBeInTheDocument();
    expect(screen.getByText("快捷操作")).toBeInTheDocument();

    expect(screen.getByText("Git / Git Bash")).toBeInTheDocument();
    expect(screen.getByText("Claude Code")).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "安装 Git for Windows" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "打开安装网络设置" }),
    ).toBeInTheDocument();
    expect(screen.getAllByRole("button", { name: "重新检测全部" })).toHaveLength(2);
  });
});
