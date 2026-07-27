import { useCallback, useEffect, useRef, useState } from "react";

import {
  bootstrap,
  commandErrorCode,
  detectTools,
  subscribeDetectionChanged,
} from "../../shared/api/client";
import type {
  AppSnapshot,
  CommandErrorCode,
  DetectionEventEnvelope,
  ToolId,
} from "../../shared/api/generated";

export type { AppSnapshot, CommandErrorCode, DetectionEventEnvelope, ToolId };

export interface DetectionApi {
  bootstrap: () => Promise<AppSnapshot>;
  detectTools: (toolIds?: ToolId[]) => Promise<string>;
  subscribeDetectionChanged: (
    listener: (event: DetectionEventEnvelope) => void,
  ) => Promise<() => void>;
}

export type SnapshotPhase = "loading" | "ready" | "bootstrapError";
export type SyncWarning = null | "eventUnavailable" | "refreshFailed";

export interface DetectionSnapshotState {
  phase: SnapshotPhase;
  snapshot: AppSnapshot | null;
  syncWarning: SyncWarning;
  detectionStartError: CommandErrorCode | null;
  refresh: () => Promise<void>;
  startDetection: (toolIds?: ToolId[]) => Promise<void>;
}

const defaultDetectionApi: DetectionApi = {
  bootstrap,
  detectTools,
  subscribeDetectionChanged,
};

export function useDetectionSnapshot(
  api: DetectionApi = defaultDetectionApi,
): DetectionSnapshotState {
  const [phase, setPhase] = useState<SnapshotPhase>("loading");
  const [snapshot, setSnapshot] = useState<AppSnapshot | null>(null);
  const [syncWarning, setSyncWarning] = useState<SyncWarning>(null);
  const [detectionStartError, setDetectionStartError] =
    useState<CommandErrorCode | null>(null);
  const snapshotRef = useRef<AppSnapshot | null>(null);
  const highestSeenSequenceRef = useRef<bigint>(0n);
  const refreshRef = useRef<Promise<void> | null>(null);
  const activeRef = useRef(true);

  const applySnapshot = useCallback((next: AppSnapshot): boolean => {
    const current = snapshotRef.current;
    if (current !== null && next.snapshotVersion < current.snapshotVersion) {
      return false;
    }

    snapshotRef.current = next;
    if (next.lastEventSequence > highestSeenSequenceRef.current) {
      highestSeenSequenceRef.current = next.lastEventSequence;
    }
    setSnapshot(next);
    setPhase("ready");
    setSyncWarning((warning) => (warning === "refreshFailed" ? null : warning));
    return true;
  }, []);

  const refresh = useCallback(async (): Promise<void> => {
    if (!activeRef.current) {
      return;
    }
    if (refreshRef.current !== null) {
      return refreshRef.current;
    }

    const refreshPromise = (async () => {
      let retryCount = 0;
      while (activeRef.current) {
        try {
          const next = await api.bootstrap();
          if (!activeRef.current) {
            return;
          }
          applySnapshot(next);
          if (highestSeenSequenceRef.current <= next.lastEventSequence) {
            return;
          }
          if (retryCount >= 1) {
            setSyncWarning("refreshFailed");
            return;
          }
          retryCount += 1;
        } catch {
          if (activeRef.current) {
            setSyncWarning("refreshFailed");
          }
          return;
        }
      }
    })();

    refreshRef.current = refreshPromise;
    try {
      await refreshPromise;
    } finally {
      if (refreshRef.current === refreshPromise) {
        refreshRef.current = null;
      }
    }
  }, [api, applySnapshot]);

  const startDetection = useCallback(
    async (toolIds?: ToolId[]): Promise<void> => {
      setDetectionStartError(null);
      try {
        await api.detectTools(toolIds);
      } catch (error) {
        if (activeRef.current) {
          setDetectionStartError(commandErrorCode(error));
        }
      }
    },
    [api],
  );

  useEffect(() => {
    activeRef.current = true;
    let unlisten: (() => void) | undefined;

    const onEvent = (event: DetectionEventEnvelope) => {
      if (!activeRef.current) {
        return;
      }
      const currentSequence = snapshotRef.current?.lastEventSequence ?? 0n;
      if (
        event.sequence <= currentSequence
        || event.sequence <= highestSeenSequenceRef.current
      ) {
        return;
      }
      highestSeenSequenceRef.current = event.sequence;
      void refresh();
    };

    const onFocus = () => {
      void refresh();
    };

    window.addEventListener("focus", onFocus);
    const initialize = async () => {
      try {
        unlisten = await api.subscribeDetectionChanged(onEvent);
        if (!activeRef.current) {
          unlisten();
          unlisten = undefined;
          return;
        }
      } catch {
        if (activeRef.current) {
          setSyncWarning("eventUnavailable");
        }
      }

      if (!activeRef.current) {
        return;
      }
      try {
        const initial = await api.bootstrap();
        if (activeRef.current) {
          applySnapshot(initial);
        }
      } catch {
        if (activeRef.current) {
          setPhase("bootstrapError");
        }
      }
    };

    void initialize();

    return () => {
      activeRef.current = false;
      window.removeEventListener("focus", onFocus);
      unlisten?.();
      unlisten = undefined;
    };
  }, [api, applySnapshot, refresh]);

  return {
    phase,
    snapshot,
    syncWarning,
    detectionStartError,
    refresh,
    startDetection,
  };
}
