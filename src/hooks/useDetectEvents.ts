import { useEffect, useRef } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { DetectResultEvent } from "../types/events";

export function useDetectEvents(
  onResult: (event: DetectResultEvent) => void,
): void {
  const onResultRef = useRef(onResult);

  useEffect(() => {
    onResultRef.current = onResult;
  }, [onResult]);

  useEffect(() => {
    let unlisten: UnlistenFn | null = null;

    void listen<DetectResultEvent>("detect:result", (event) => {
      onResultRef.current(event.payload);
    }).then((dispose) => {
      unlisten = dispose;
    });

    return () => {
      if (unlisten) {
        void unlisten();
      }
    };
  }, []);
}
