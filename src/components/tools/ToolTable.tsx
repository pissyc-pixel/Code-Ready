import StatusBadge from "../ui/StatusBadge";
import type { InstallTaskViewState } from "../../types/install";
import type { ToolId, ToolStatus } from "../../types/tool";

const INSTALLABLE_TOOL_IDS = new Set<ToolId>([
  "git",
  "node",
  "python",
  "claude",
  "codex",
  "opencode",
  "ccswitch",
]);

const LATEST_INSTALLABLE_TOOL_IDS = new Set<ToolId>([
  "claude",
  "codex",
  "opencode",
]);

type ToolTableProps = {
  rows: ToolStatus[];
  installState: InstallTaskViewState;
  subtitles?: Partial<Record<ToolId, string>>;
  getRowNote?: (row: ToolStatus) => string | undefined;
  onDetect: (toolId: ToolId) => Promise<void>;
  onInstall: (toolId: ToolId) => Promise<void>;
  onReinstall: (toolId: ToolId) => Promise<void>;
  onInstallLatest: (toolId: ToolId) => Promise<void>;
  onCancelInstall: () => Promise<void>;
  onOpenPathRepair: (tool: ToolStatus) => void;
  onOpenLogDirectory?: () => Promise<void>;
  onNavigateLogs?: () => void;
};

function ToolTable({
  rows,
  installState,
  subtitles,
  getRowNote,
  onDetect,
  onInstall,
  onReinstall,
  onInstallLatest,
  onCancelInstall,
  onOpenPathRepair,
  onOpenLogDirectory,
  onNavigateLogs,
}: ToolTableProps) {
  return (
    <div className="table-wrap">
      <table>
        <thead>
          <tr>
            <th>工具</th>
            <th>状态</th>
            <th>版本</th>
            <th>路径</th>
            <th>操作</th>
          </tr>
        </thead>
        <tbody>
          {rows.map((row) => {
            const subtitle = subtitles?.[row.id];
            const canInstall = INSTALLABLE_TOOL_IDS.has(row.id);
            const canInstallLatest = LATEST_INSTALLABLE_TOOL_IDS.has(row.id);
            const isActiveInstall =
              installState.isInstalling && installState.activeToolId === row.id;
            const isOtherInstallRunning =
              installState.isInstalling && installState.activeToolId !== row.id;
            const showInstallButton = canInstall && row.status === "missing";
            const showReinstallButton =
              canInstall &&
              row.status !== "missing" &&
              row.status !== "checking" &&
              !isActiveInstall;
            const showInstallLatestButton =
              canInstallLatest && row.status !== "checking" && !isActiveInstall;
            const showCancelButton = isActiveInstall;
            const showPathRepairButton =
              row.status === "installed_but_path_missing" &&
              Boolean(row.executablePath);
            const showLogButtons =
              row.status === "installed_but_path_missing" ||
              row.status === "detect_failed" ||
              row.status === "install_failed" ||
              row.status === "broken";
            const note = getRowNote?.(row);

            return (
              <tr key={row.id}>
                <td>
                  <div className="tool-name-stack">
                    <div className="tool-name">{row.name}</div>
                    {subtitle ? <div className="tool-subtitle">{subtitle}</div> : null}
                  </div>
                </td>
                <td>
                  <StatusBadge status={row.status} />
                </td>
                <td className="mono">{row.version ?? "—"}</td>
                <td>
                  <div className="path-cell" title={row.executablePath ?? "—"}>
                    {row.executablePath ?? "—"}
                  </div>
                </td>
                <td>
                  {row.status === "checking" ? (
                    <span className="checking-text">等待结果...</span>
                  ) : (
                    <div className="action-group">
                      {showInstallButton ? (
                        <button
                          type="button"
                          className="ghost-button"
                          disabled={isOtherInstallRunning}
                          onClick={() => void onInstall(row.id)}
                        >
                          安装
                        </button>
                      ) : null}
                      {showReinstallButton ? (
                        <button
                          type="button"
                          className="ghost-button"
                          disabled={isOtherInstallRunning}
                          onClick={() => void onReinstall(row.id)}
                        >
                          重装
                        </button>
                      ) : null}
                      {showInstallLatestButton ? (
                        <button
                          type="button"
                          className="ghost-button"
                          disabled={isOtherInstallRunning}
                          onClick={() => void onInstallLatest(row.id)}
                        >
                          安装最新版
                        </button>
                      ) : null}
                      {showCancelButton ? (
                        <button type="button" className="ghost-button" onClick={() => void onCancelInstall()}>
                          取消安装
                        </button>
                      ) : null}
                      {showPathRepairButton ? (
                        <button
                          type="button"
                          className="ghost-button"
                          onClick={() => onOpenPathRepair(row)}
                        >
                          查看 PATH 修复说明
                        </button>
                      ) : null}
                      {showLogButtons && onNavigateLogs ? (
                        <button type="button" className="ghost-button" onClick={onNavigateLogs}>
                          查看日志
                        </button>
                      ) : null}
                      {showLogButtons && onOpenLogDirectory ? (
                        <button
                          type="button"
                          className="ghost-button"
                          onClick={() => void onOpenLogDirectory()}
                        >
                          打开日志目录
                        </button>
                      ) : null}
                      <button
                        type="button"
                        className="ghost-button"
                        onClick={() => void onDetect(row.id)}
                      >
                        重新检测
                      </button>
                    </div>
                  )}
                  {note ? <div className={`row-message row-message-${row.status}`}>{note}</div> : null}
                </td>
              </tr>
            );
          })}
        </tbody>
      </table>
    </div>
  );
}

export type { ToolTableProps };
export default ToolTable;
