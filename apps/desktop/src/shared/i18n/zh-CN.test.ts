import { describe, expect, test } from "vitest";

import { factMessageKey, versionMessageKey } from "../../features/detection/presentation";
import { formatObservedVersion, toolLabel } from "./zh-CN";

test.each([
  ["absent", "status.absent"],
  ["presentHealthy", "status.presentHealthy"],
  ["presentPathIssue", "status.presentPathIssue"],
  ["presentBroken", "status.presentBroken"],
  ["unknown", "status.unknown"],
] as const)("maps fact %s without inspecting version state", (state, key) => {
  expect(factMessageKey(state)).toBe(key);
});

test.each([
  ["current", "version.current"],
  ["outdated", "version.outdated"],
  ["newerThanKnown", "version.newerThanKnown"],
  ["notComparable", "version.notComparable"],
  ["unknown", "version.unknown"],
] as const)("maps version status %s separately", (state, key) => {
  expect(versionMessageKey(state)).toBe(key);
});

test("translates every registry tool label used by slice one", () => {
  for (const key of [
    "tools.git.name",
    "tools.claudeCode.name",
    "tools.codexCli.name",
  ]) {
    expect(toolLabel(key)).not.toEqual(key);
  }
  expect(toolLabel("tools.notInSliceOne.name")).toBe("未知工具");
});

test("formats a parsed version locally", () => {
  expect(formatObservedVersion("0.138.0")).toBe("版本 0.138.0");
});

describe("i18n module exports", () => {
  test("keeps the stable message function typed at runtime", async () => {
    const { message } = await import("./zh-CN");
    expect(message("onboarding.start")).toBe("开始检测");
  });
});
