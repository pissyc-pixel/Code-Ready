import { statusMeta } from "../../lib/toolStatus";
import type { ToolInstallStatus } from "../../types/tool";

type StatusBadgeProps = {
  status: ToolInstallStatus;
};

function StatusBadge({ status }: StatusBadgeProps) {
  const meta = statusMeta[status];
  return <span className={`status-badge ${meta.className}`}>{meta.label}</span>;
}

export default StatusBadge;
