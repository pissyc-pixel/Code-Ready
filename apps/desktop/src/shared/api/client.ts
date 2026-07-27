import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

import type {
  AppSnapshot,
  CommandError,
  CommandErrorCode,
  DetectionEventEnvelope,
  ToolId,
} from "./generated";

const COMMAND_ERROR_CODES: readonly CommandErrorCode[] = [
  "invalidToolSelection",
  "detectionAlreadyRunning",
  "internal",
];

export function bootstrap(): Promise<AppSnapshot> {
  return invoke<AppSnapshot>("bootstrap");
}

export function detectTools(toolIds?: ToolId[]): Promise<string> {
  return invoke<string>("detect_tools", { toolIds });
}

export function subscribeDetectionChanged(
  listener: (event: DetectionEventEnvelope) => void,
): Promise<UnlistenFn> {
  return listen<DetectionEventEnvelope>("detection.changed", ({ payload }) => {
    listener(payload);
  });
}

export function commandErrorCode(error: unknown): CommandErrorCode {
  if (typeof error !== "object" || error === null || !("code" in error)) {
    return "internal";
  }

  const code = (error as Partial<CommandError>).code;
  if (typeof code === "string" && COMMAND_ERROR_CODES.includes(code as CommandErrorCode)) {
    return code as CommandErrorCode;
  }
  return "internal";
}
