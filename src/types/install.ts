import type { ToolId, ToolInstallStatus } from "./tool";

export type InstallPhase =
  | "started"
  | "running"
  | "success"
  | "failed"
  | "cancelled"
  | "timeout";

export type InstallResult = {
  exitCode?: number;
  durationMs?: number;
};

export type InstallProgressEvent = {
  toolId: ToolId;
  phase: "running";
  line: string;
  stream: "stdout" | "stderr";
  timestamp: string;
};

export type InstallStatusEvent = {
  toolId: ToolId;
  status: ToolInstallStatus;
  phase: InstallPhase;
  errorMessage?: string;
  suggestion?: string;
  result?: InstallResult;
  timestamp: string;
};

export type InstallTaskViewState = {
  activeToolId?: ToolId;
  isInstalling: boolean;
};
