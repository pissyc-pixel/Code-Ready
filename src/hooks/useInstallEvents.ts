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
    let unlistenProgress: UnlistenFn | null = null;
    let unlistenStatus: UnlistenFn | null = null;

    void listen<InstallProgressEvent>("install:progress", (event) => {
      optionsRef.current.onProgress?.(event.payload);
    }).then((dispose) => {
      unlistenProgress = dispose;
    });

    void listen<InstallStatusEvent>("install:status", (event) => {
      optionsRef.current.onStatus?.(event.payload);
    }).then((dispose) => {
      unlistenStatus = dispose;
    });

    return () => {
      if (unlistenProgress) {
        void unlistenProgress();
      }
      if (unlistenStatus) {
        void unlistenStatus();
      }
    };
  }, []);
}
