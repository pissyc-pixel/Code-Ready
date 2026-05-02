import { useEffect } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { DetectResultEvent } from "../types/events";

type UseDetectEventsOptions = {
  onResult: (event: DetectResultEvent) => void;
};

export function useDetectEvents(options: UseDetectEventsOptions): void {
  useEffect(() => {
    let unlisten: UnlistenFn | null = null;

    void listen<DetectResultEvent>("detect:result", (event) => {
      options.onResult(event.payload);
    }).then((dispose) => {
      unlisten = dispose;
    });

    return () => {
      if (unlisten) {
        void unlisten();
      }
    };
  }, [options]);
}
