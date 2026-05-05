import { useEffect, useRef } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { InstallProgressEvent, InstallStatusEvent } from "../types/install";

type UseInstallEventsOptions = {
  onProgress?: (event: InstallProgressEvent) => void;
  onStatus?: (event: InstallStatusEvent) => void;
};

export function useInstallEvents(options: UseInstallEventsOptions): void {
  const optionsRef = useRef(options);

  useEffect(() => {
    optionsRef.current = options;
  }, [options]);

  useEffect(() => {
    let cancelled = false;
    let unlistenProgress: UnlistenFn | undefined;
    let unlistenStatus: UnlistenFn | undefined;

    listen<InstallProgressEvent>("install:progress", (event) => {
      optionsRef.current.onProgress?.(event.payload);
    }).then((dispose) => {
      if (cancelled) {
        dispose();
        return;
      }
      unlistenProgress = dispose;
    });

    listen<InstallStatusEvent>("install:status", (event) => {
      optionsRef.current.onStatus?.(event.payload);
    }).then((dispose) => {
      if (cancelled) {
        dispose();
        return;
      }
      unlistenStatus = dispose;
    });

    return () => {
      cancelled = true;
      unlistenProgress?.();
      unlistenStatus?.();
    };
  }, []);
}
