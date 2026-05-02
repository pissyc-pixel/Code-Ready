import type { ToolId, ToolStatus } from "./tool";

export type DetectResultEvent = {
  toolId: ToolId;
  result: ToolStatus;
};
