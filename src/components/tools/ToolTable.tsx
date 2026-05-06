import type { ReactNode } from "react";
import { getToolVersionLabel } from "../../lib/toolStatus";
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
  layout?: "table" | "cards";
  onDetect: (toolId: ToolId) => Promise<void>;
  onInstall: (toolId: ToolId) => Promise<void>;
  onReinstall: (toolId: ToolId) => Promise<void>;
  onInstallLatest: (toolId: ToolId) => Promise<void>;
  onCancelInstall: () => Promise<void>;
  onOpenPathRepair: (tool: ToolStatus) => void;
  onOpenLogDirectory?: () => Promise<void>;
  onNavigateLogs?: () => void;
};

type ToolRowState = ReturnType<typeof buildToolRowState>;

function ToolTable({
  rows,
  installState,
  subtitles,
  getRowNote,
  layout = "table",
  onDetect,
  onInstall,
  onReinstall,
  onInstallLatest,
  onCancelInstall,
  onOpenPathRepair,
  onOpenLogDirectory,
  onNavigateLogs,
}: ToolTableProps) {
  const rowStates = rows.map((row) =>
    buildToolRowState(row, installState, getRowNote?.(row)),
  );

  if (layout === "cards") {
    return (
      <div className="tool-card-list">
        {rowStates.map((rowState) => (
          <article key={rowState.row.id} className="tool-card">
            <div className="tool-card-header">
              <div className="tool-name-stack">
                <div className="tool-name">{rowState.row.name}</div>
                {subtitles?.[rowState.row.id] ? (
                  <div className="tool-subtitle">{subtitles[rowState.row.id]}</div>
                ) : null}
              </div>
              <StatusBadge status={rowState.row.status} />
            </div>

            <div className="tool-card-grid">
              <div className="tool-card-field">
                <span>版本</span>
                <strong className="mono">{getToolVersionLabel(rowState.row)}</strong>
              </div>
              <div className="tool-card-field tool-card-field-wide">
                <span>路径</span>
                <strong className="mono tool-card-path">
                  {rowState.row.executablePath ?? "—"}
                </strong>
              </div>
            </div>

            {rowState.note ? (
              <div className={`row-message row-message-${rowState.row.status}`}>
                {rowState.note}
              </div>
            ) : null}

            <div className="tool-card-actions">
              {renderToolActions(
                rowState,
                onDetect,
                onInstall,
                onReinstall,
                onInstallLatest,
                onCancelInstall,
                onOpenPathRepair,
                onOpenLogDirectory,
                onNavigateLogs,
              )}
            </div>
          </article>
        ))}
      </div>
    );
  }

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
          {rowStates.map((rowState) => (
            <tr key={rowState.row.id}>
              <td>
                <div className="tool-name-stack">
                  <div className="tool-name">{rowState.row.name}</div>
                  {subtitles?.[rowState.row.id] ? (
                    <div className="tool-subtitle">{subtitles[rowState.row.id]}</div>
                  ) : null}
                </div>
              </td>
              <td>
                <StatusBadge status={rowState.row.status} />
              </td>
              <td className="mono">{getToolVersionLabel(rowState.row)}</td>
              <td>
                <div className="path-cell" title={rowState.row.executablePath ?? "—"}>
                  {rowState.row.executablePath ?? "—"}
                </div>
              </td>
              <td>
                {rowState.row.status === "checking" ? (
                  <span className="checking-text">等待结果...</span>
                ) : (
                  <div className="action-group">
                    {renderToolActions(
                      rowState,
                      onDetect,
                      onInstall,
                      onReinstall,
                      onInstallLatest,
                      onCancelInstall,
                      onOpenPathRepair,
                      onOpenLogDirectory,
                      onNavigateLogs,
                    )}
                  </div>
                )}
                {rowState.note ? (
                  <div className={`row-message row-message-${rowState.row.status}`}>
                    {rowState.note}
                  </div>
                ) : null}
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

function buildToolRowState(
  row: ToolStatus,
  installState: InstallTaskViewState,
  note?: string,
) {
  const canInstall = INSTALLABLE_TOOL_IDS.has(row.id);
  const canInstallLatest = LATEST_INSTALLABLE_TOOL_IDS.has(row.id);
  const isActiveInstall = installState.isInstalling && installState.activeToolId === row.id;
  const isOtherInstallRunning =
    installState.isInstalling && installState.activeToolId !== row.id;

  return {
    row,
    note,
    isActiveInstall,
    isOtherInstallRunning,
    showInstallButton: canInstall && row.status === "missing",
    showReinstallButton:
      canInstall &&
      row.status !== "missing" &&
      row.status !== "checking" &&
      !isActiveInstall,
    showInstallLatestButton:
      canInstallLatest && row.status !== "checking" && !isActiveInstall,
    showCancelButton: isActiveInstall,
    showPathRepairButton:
      row.status === "installed_but_path_missing" && Boolean(row.executablePath),
    showLogButtons:
      row.status === "installed_but_path_missing" ||
      row.status === "detect_failed" ||
      row.status === "install_failed" ||
      row.status === "broken",
  };
}

function renderToolActions(
  rowState: ToolRowState,
  onDetect: (toolId: ToolId) => Promise<void>,
  onInstall: (toolId: ToolId) => Promise<void>,
  onReinstall: (toolId: ToolId) => Promise<void>,
  onInstallLatest: (toolId: ToolId) => Promise<void>,
  onCancelInstall: () => Promise<void>,
  onOpenPathRepair: (tool: ToolStatus) => void,
  onOpenLogDirectory?: () => Promise<void>,
  onNavigateLogs?: () => void,
): ReactNode[] {
  const actions: ReactNode[] = [];

  if (rowState.row.status === "checking") {
    return [<span key="checking" className="checking-text">等待结果...</span>];
  }

  if (rowState.showInstallButton) {
    actions.push(
      <button
        key="install"
        type="button"
        className="ghost-button"
        disabled={rowState.isOtherInstallRunning}
        onClick={() => void onInstall(rowState.row.id)}
      >
        安装
      </button>,
    );
  }

  if (rowState.showReinstallButton) {
    actions.push(
      <button
        key="reinstall"
        type="button"
        className="ghost-button"
        disabled={rowState.isOtherInstallRunning}
        onClick={() => void onReinstall(rowState.row.id)}
      >
        重装
      </button>,
    );
  }

  if (rowState.showInstallLatestButton) {
    actions.push(
      <button
        key="latest"
        type="button"
        className="ghost-button"
        disabled={rowState.isOtherInstallRunning}
        onClick={() => void onInstallLatest(rowState.row.id)}
      >
        安装最新版
      </button>,
    );
  }

  if (rowState.showCancelButton) {
    actions.push(
      <button key="cancel" type="button" className="ghost-button" onClick={() => void onCancelInstall()}>
        取消安装
      </button>,
    );
  }

  if (rowState.showPathRepairButton) {
    actions.push(
      <button
        key="path-repair"
        type="button"
        className="ghost-button"
        onClick={() => onOpenPathRepair(rowState.row)}
      >
        查看 PATH 修复说明
      </button>,
    );
  }

  if (rowState.showLogButtons && onNavigateLogs) {
    actions.push(
      <button key="logs" type="button" className="ghost-button" onClick={onNavigateLogs}>
        查看日志
      </button>,
    );
  }

  if (rowState.showLogButtons && onOpenLogDirectory) {
    actions.push(
      <button
        key="log-dir"
        type="button"
        className="ghost-button"
        onClick={() => void onOpenLogDirectory()}
      >
        打开日志目录
      </button>,
    );
  }

  actions.push(
    <button
      key="detect"
      type="button"
      className="ghost-button"
      onClick={() => void onDetect(rowState.row.id)}
    >
      重新检测
    </button>,
  );

  return actions;
}

export type { ToolTableProps };
export default ToolTable;
