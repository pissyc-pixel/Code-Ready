import type { ToolId, ToolStatus } from "./tool";
import type {
  InstallProgressEvent,
  InstallStatusEvent,
} from "./install";

export type DetectResultEvent = {
  toolId: ToolId;
  result: ToolStatus;
};

export type { InstallProgressEvent, InstallStatusEvent };
