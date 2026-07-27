import { act, cleanup, render, screen } from "@testing-library/react";
import { StrictMode } from "react";
import { afterEach, describe, expect, test, vi } from "vitest";

import type {
  AppSnapshot,
  DetectionEventEnvelope,
  DetectionApi,
  ToolId,
} from "./useDetectionSnapshot";
import { useDetectionSnapshot } from "./useDetectionSnapshot";

function snapshot(snapshotVersion: number, lastEventSequence: number): AppSnapshot {
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

function event(sequence: number, snapshotVersion: number): DetectionEventEnvelope {
  return {
    schemaVersion: 1,
    sequence,
    snapshotVersion,
    emittedAtEpochMs: 1_754_000_000_000,
    eventType: "detection.changed",
    runId: "run-1",
  };
}

type Deferred<T> = {
  promise: Promise<T>;
  resolve: (value: T) => void;
};

function deferred<T>(): Deferred<T> {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((resolvePromise) => {
    resolve = resolvePromise;
  });
  return { promise, resolve };
}

class FakeApi implements DetectionApi {
  calls: string[] = [];
  detectedToolIds: ToolId[] | undefined;
  subscriptionError = false;
  detectionError: unknown;
  unlistenCount = 0;
  private snapshots: Array<AppSnapshot | Promise<AppSnapshot>> = [];
  private bootstrapResults: Array<AppSnapshot | Promise<AppSnapshot> | Error> = [];
  private listener: ((event: DetectionEventEnvelope) => void) | null = null;

  withSnapshots(...snapshots: AppSnapshot[]): this {
    this.snapshots = snapshots;
    return this;
  }

  withDeferredSnapshots(...snapshots: Array<Promise<AppSnapshot>>): this {
    this.snapshots = snapshots;
    return this;
  }

  withBootstrapResults(
    ...results: Array<AppSnapshot | Promise<AppSnapshot> | Error>
  ): this {
    this.bootstrapResults = results;
    return this;
  }

  bootstrap(): Promise<AppSnapshot> {
    this.calls.push("bootstrap");
    const next = this.bootstrapResults.shift() ?? this.snapshots.shift();
    if (next instanceof Error) {
      return Promise.reject(next);
    }
    return Promise.resolve(next ?? snapshot(1, 0));
  }

  detectTools(toolIds?: ToolId[]): Promise<string> {
    this.calls.push("detect");
    this.detectedToolIds = toolIds;
    if (this.detectionError !== undefined) {
      return Promise.reject(this.detectionError);
    }
    return Promise.resolve("run-1");
  }

  subscribeDetectionChanged(
    listener: (event: DetectionEventEnvelope) => void,
  ): Promise<() => void> {
    this.calls.push("subscribe");
    if (this.subscriptionError) {
      return Promise.reject(new Error("subscription unavailable"));
    }
    this.listener = listener;
    return Promise.resolve(() => {
      this.unlistenCount += 1;
      this.listener = null;
    });
  }

  emit(next: DetectionEventEnvelope): void {
    this.listener?.(next);
  }

  bootstrapCallCount(): number {
    return this.calls.filter((call) => call === "bootstrap").length;
  }
}

function HookProbe({ api }: { api: DetectionApi }) {
  const state = useDetectionSnapshot(api);
  const version = state.snapshot?.snapshotVersion.toString() ?? "none";
  const sequence = state.snapshot?.lastEventSequence.toString() ?? "none";
  return (
    <>
      <output>{state.phase}:{version}:{sequence}:{state.syncWarning ?? "none"}</output>
      <button type="button" onClick={() => void state.refresh()}>refresh</button>
    </>
  );
}

function StartProbe({ api }: { api: DetectionApi }) {
  const state = useDetectionSnapshot(api);
  return (
    <>
      <output>{state.detectionStartError ?? "none"}</output>
      <button type="button" onClick={() => void state.startDetection(["git"])}>
        detect
      </button>
    </>
  );
}

function StartRefreshProbe({ api }: { api: DetectionApi }) {
  const state = useDetectionSnapshot(api);
  return (
    <>
      <output>{state.detectionStartError ?? "none"}:{state.snapshot?.snapshotVersion.toString() ?? "none"}</output>
      <button type="button" onClick={() => void state.startDetection(["git"])}>
        detect
      </button>
    </>
  );
}

describe("useDetectionSnapshot", () => {
  afterEach(() => {
    cleanup();
  });

  test("subscribes before bootstrap and applies the returned snapshot", async () => {
    const api = new FakeApi().withSnapshots(snapshot(1, 0));
    render(<HookProbe api={api} />);

    expect(await screen.findByText("ready:1:0:none")).toBeInTheDocument();
    expect(api.calls).toEqual(["subscribe", "bootstrap"]);
  });

  test("coalesces duplicate events and refreshes from snapshot truth", async () => {
    const api = new FakeApi().withSnapshots(snapshot(1, 0), snapshot(3, 2));
    render(<HookProbe api={api} />);
    await screen.findByText("ready:1:0:none");

    act(() => {
      api.emit(event(2, 3));
      api.emit(event(2, 3));
    });

    expect(await screen.findByText("ready:3:2:none")).toBeInTheDocument();
    expect(api.bootstrapCallCount()).toBe(2);
  });

  test("a sequence gap causes bootstrap instead of event replay", async () => {
    const api = new FakeApi().withSnapshots(snapshot(4, 3), snapshot(8, 7));
    render(<HookProbe api={api} />);
    await screen.findByText("ready:4:3:none");

    act(() => api.emit(event(7, 8)));

    expect(await screen.findByText("ready:8:7:none")).toBeInTheDocument();
    expect(api.bootstrapCallCount()).toBe(2);
  });

  test("never applies a slower older snapshot over a newer snapshot", async () => {
    const first = deferred<AppSnapshot>();
    const second = deferred<AppSnapshot>();
    const api = new FakeApi().withDeferredSnapshots(first.promise, second.promise);
    render(<HookProbe api={api} />);

    act(() => window.dispatchEvent(new Event("focus")));
    second.resolve(snapshot(5, 4));
    expect(await screen.findByText("ready:5:4:none")).toBeInTheDocument();
    first.resolve(snapshot(1, 0));

    expect(screen.getByText("ready:5:4:none")).toBeInTheDocument();
  });

  test("window focus reloads the snapshot even when no event arrived", async () => {
    const api = new FakeApi().withSnapshots(snapshot(1, 0), snapshot(2, 1));
    render(<HookProbe api={api} />);
    await screen.findByText("ready:1:0:none");

    act(() => window.dispatchEvent(new Event("focus")));

    expect(await screen.findByText("ready:2:1:none")).toBeInTheDocument();
  });

  test("unmount calls the event unlisten exactly once", async () => {
    const api = new FakeApi().withSnapshots(snapshot(1, 0));
    const view = render(<HookProbe api={api} />);
    await screen.findByText("ready:1:0:none");

    view.unmount();

    expect(api.unlistenCount).toBe(1);
  });

  test("subscription rejection still bootstraps a snapshot with a warning", async () => {
    const api = new FakeApi().withSnapshots(snapshot(1, 0));
    api.subscriptionError = true;
    render(<HookProbe api={api} />);

    expect(await screen.findByText("ready:1:0:eventUnavailable")).toBeInTheDocument();
  });

  test("an initial bootstrap rejection can recover through refresh", async () => {
    const api = new FakeApi().withBootstrapResults(
      new Error("bootstrap unavailable"),
      snapshot(2, 1),
    );
    render(<HookProbe api={api} />);

    expect(await screen.findByText("bootstrapError:none:none:none")).toBeInTheDocument();
    await act(async () => {
      screen.getByRole("button", { name: "refresh" }).click();
    });

    expect(await screen.findByText("ready:2:1:none")).toBeInTheDocument();
  });

  test("a failed refresh preserves the old snapshot and shows a warning", async () => {
    const api = new FakeApi().withBootstrapResults(snapshot(1, 0), new Error("refresh failed"));
    render(<HookProbe api={api} />);
    await screen.findByText("ready:1:0:none");

    act(() => api.emit(event(1, 2)));

    expect(await screen.findByText("ready:1:0:refreshFailed")).toBeInTheDocument();
  });

  test("refreshes snapshot truth after an accepted detection command", async () => {
    const api = new FakeApi().withSnapshots(snapshot(1, 0), snapshot(2, 1));
    render(<StartRefreshProbe api={api} />);
    expect(await screen.findByText("none:1")).toBeInTheDocument();

    await act(async () => {
      screen.getByRole("button", { name: "detect" }).click();
    });

    expect(await screen.findByText("none:2")).toBeInTheDocument();
    expect(api.calls).toEqual(["subscribe", "bootstrap", "detect", "bootstrap"]);
  });

  test("startDetection forwards the selection and maps a rejected command", async () => {
    const api = new FakeApi().withSnapshots(snapshot(1, 0));
    api.detectionError = { code: "detectionAlreadyRunning", retryable: true };
    render(<StartProbe api={api} />);
    await screen.findByText("none");

    await act(async () => {
      screen.getByRole("button", { name: "detect" }).click();
    });

    expect(api.detectedToolIds).toEqual(["git"]);
    expect(screen.getByText("detectionAlreadyRunning")).toBeInTheDocument();
  });

  test("cleans up an async subscription that resolves after StrictMode cleanup", async () => {
    const firstSubscription = deferred<() => void>();
    const secondSubscription = deferred<() => void>();
    const firstUnlisten = vi.fn();
    const secondUnlisten = vi.fn();
    let subscriptionCalls = 0;
    const api: DetectionApi = {
      bootstrap: () => Promise.resolve(snapshot(1, 0)),
      detectTools: () => Promise.resolve("run-1"),
      subscribeDetectionChanged: () => {
        subscriptionCalls += 1;
        return subscriptionCalls === 1
          ? firstSubscription.promise
          : secondSubscription.promise;
      },
    };

    const view = render(
      <StrictMode>
        <HookProbe api={api} />
      </StrictMode>,
    );
    await act(async () => {
      firstSubscription.resolve(firstUnlisten);
      secondSubscription.resolve(secondUnlisten);
      await Promise.resolve();
    });
    view.unmount();

    expect(firstUnlisten).toHaveBeenCalledOnce();
    expect(secondUnlisten).toHaveBeenCalledOnce();
  });
});
