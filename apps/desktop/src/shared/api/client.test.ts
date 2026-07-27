import { beforeEach, describe, expect, test, vi } from "vitest";

import type { AppSnapshot } from "./generated";
import { bootstrap, detectTools, subscribeDetectionChanged } from "./client";

import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));
vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(),
}));

const mockedInvoke = vi.mocked(invoke);
const mockedListen = vi.mocked(listen);

function snapshot(snapshotVersion: bigint, lastEventSequence: bigint): AppSnapshot {
  return {
    schemaVersion: 2,
    snapshotVersion,
    lastEventSequence,
    platform: "macosArm64",
    tools: [],
    observations: [],
    detectionRun: null,
  };
}

describe("detection API client", () => {
  beforeEach(() => {
    mockedInvoke.mockReset();
    mockedListen.mockReset();
  });

  test("uses only the two narrow commands and stable event name", async () => {
    mockedInvoke
      .mockResolvedValueOnce(snapshot(1n, 0n))
      .mockResolvedValueOnce("run-1")
      .mockResolvedValueOnce("run-2");
    mockedListen.mockResolvedValue(() => undefined);

    await bootstrap();
    await detectTools();
    await detectTools(["git"]);
    await subscribeDetectionChanged(() => undefined);

    expect(mockedInvoke).toHaveBeenNthCalledWith(1, "bootstrap");
    expect(mockedInvoke).toHaveBeenNthCalledWith(2, "detect_tools", { toolIds: undefined });
    expect(mockedInvoke).toHaveBeenNthCalledWith(3, "detect_tools", { toolIds: ["git"] });
    expect(mockedListen).toHaveBeenCalledWith("detection.changed", expect.any(Function));
  });
});
