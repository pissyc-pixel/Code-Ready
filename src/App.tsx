import { useEffect, useMemo, useState } from "react";
import "./App.css";
import { mockAiTools, mockBaseTools } from "./data/mockTools";
import { useDetectEvents } from "./hooks/useDetectEvents";
import { useInstallEvents } from "./hooks/useInstallEvents";
import {
  cancelInstall,
  detectAllTools,
  detectTool,
  getConfig,
  installLatestTool,
  installTool,
  isAdmin,
  openCcSwitch,
  reinstallTool,
  restartAsAdmin,
  updateConfig,
} from "./lib/api";
import { statusMeta } from "./lib/toolStatus";
import type { AppConfig } from "./types/config";
import { defaultAppConfig } from "./types/config";
import type {
  InstallStatusEvent,
  InstallTaskViewState,
} from "./types/install";
import type { ToolId, ToolStatus } from "./types/tool";

const initialRows = [...mockBaseTools, ...mockAiTools];
const INSTALLABLE_TOOL_IDS: ToolId[] = [
  "git",
  "node",
  "python",
  "claude",
  "codex",
  "opencode",
  "ccswitch",
];
const LATEST_INSTALLABLE_TOOL_IDS: ToolId[] = ["claude", "codex", "opencode"];

function App() {
  const [rows, setRows] = useState<ToolStatus[]>(() =>
    initialRows.map((tool) => ({ ...tool, status: "checking" })),
  );
  const [isDetectingAll, setIsDetectingAll] = useState(false);
  const [installState, setInstallState] = useState<InstallTaskViewState>({
    isInstalling: false,
  });
  const [config, setConfig] = useState<AppConfig>(defaultAppConfig);
  const [ccswitchPathInput, setCcswitchPathInput] = useState("");
  const [pathRepairTool, setPathRepairTool] = useState<ToolStatus | null>(null);
  const [pathRepairCopyState, setPathRepairCopyState] = useState<
    "idle" | "copied" | "failed"
  >("idle");
  const [adminState, setAdminState] = useState<{
    checked: boolean;
    isAdmin: boolean;
  }>({
    checked: false,
    isAdmin: false,
  });

  useDetectEvents((event) => {
    applyResult(event.result);
  });

  useInstallEvents({
    onStatus: applyInstallStatus,
  });

  useEffect(() => {
    void runDetectAll();
    void loadConfig();
    void loadPrivilege();
  }, []);

  const groupedRows = useMemo(
    () => ({
      base: rows.filter((row) => row.category === "base"),
      ai: rows.filter((row) => row.category === "ai"),
    }),
    [rows],
  );
  const ccswitchRow = rows.find((row) => row.id === "ccswitch");

  function applyResult(result: ToolStatus) {
    setRows((currentRows) =>
      currentRows.map((row) => {
        if (row.id !== result.id) {
          return row;
        }

        if (
          row.lastCheckedAt &&
          result.lastCheckedAt &&
          row.lastCheckedAt > result.lastCheckedAt
        ) {
          return row;
        }

        return result;
      }),
    );
  }

  function applyInstallStatus(event: InstallStatusEvent) {
    setRows((currentRows) =>
      currentRows.map((row) =>
        row.id === event.toolId
          ? {
              ...row,
              status: event.status,
              errorMessage: event.errorMessage,
              suggestion: event.suggestion ?? row.suggestion,
              lastCheckedAt: event.timestamp,
            }
          : row,
      ),
    );

    setInstallState({
      activeToolId:
        event.phase === "started" || event.phase === "running"
          ? event.toolId
          : undefined,
      isInstalling: event.phase === "started" || event.phase === "running",
    });
  }

  async function loadConfig() {
    const nextConfig = await getConfig();
    setConfig(nextConfig);
    setCcswitchPathInput(nextConfig.ccswitchPath ?? "");
  }

  async function loadPrivilege() {
    const nextIsAdmin = await isAdmin();
    setAdminState({
      checked: true,
      isAdmin: nextIsAdmin,
    });
  }

  async function runDetectAll() {
    setIsDetectingAll(true);
    setRows((currentRows) =>
      currentRows.map((row) => ({
        ...row,
        status: "checking",
        errorMessage: undefined,
      })),
    );

    try {
      const results = await detectAllTools();
      results.forEach((result) => {
        applyResult(result);
      });
    } finally {
      setIsDetectingAll(false);
    }
  }

  async function runDetectOne(toolId: ToolId) {
    setRows((currentRows) =>
      currentRows.map((row) =>
        row.id === toolId
          ? { ...row, status: "checking", errorMessage: undefined }
          : row,
      ),
    );

    const result = await detectTool(toolId);
    applyResult(result);
  }

  async function runInstall(toolId: ToolId) {
    try {
      await installTool(toolId);
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      setRows((currentRows) =>
        currentRows.map((row) =>
          row.id === toolId ? { ...row, errorMessage: message } : row,
        ),
      );
    }
  }

  async function runReinstall(toolId: ToolId) {
    try {
      await reinstallTool(toolId);
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      setRows((currentRows) =>
        currentRows.map((row) =>
          row.id === toolId ? { ...row, errorMessage: message } : row,
        ),
      );
    }
  }

  async function runInstallLatest(toolId: ToolId) {
    try {
      await installLatestTool(toolId);
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      setRows((currentRows) =>
        currentRows.map((row) =>
          row.id === toolId ? { ...row, errorMessage: message } : row,
        ),
      );
    }
  }

  async function runCancelInstall() {
    try {
      await cancelInstall();
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      setRows((currentRows) =>
        currentRows.map((row) =>
          row.id === installState.activeToolId
            ? { ...row, errorMessage: message }
            : row,
        ),
      );
    }
  }

  async function runRestartAsAdmin() {
    await restartAsAdmin();
  }

  async function saveCcSwitchPath() {
    const nextConfig = await updateConfig({
      ccswitchPath: ccswitchPathInput,
    });
    setConfig(nextConfig);
    setCcswitchPathInput(nextConfig.ccswitchPath ?? "");
    const result = await detectTool("ccswitch");
    applyResult(result);
  }

  async function runOpenCcSwitch() {
    try {
      await openCcSwitch();
      setRows((currentRows) =>
        currentRows.map((row) =>
          row.id === "ccswitch" ? { ...row, errorMessage: undefined } : row,
        ),
      );
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      setRows((currentRows) =>
        currentRows.map((row) =>
          row.id === "ccswitch" ? { ...row, errorMessage: message } : row,
        ),
      );
    }
  }

  async function copyPathRepairCommand(command: string) {
    try {
      await navigator.clipboard.writeText(command);
      setPathRepairCopyState("copied");
    } catch {
      setPathRepairCopyState("failed");
    }
  }

  function openPathRepair(tool: ToolStatus) {
    setPathRepairTool(tool);
    setPathRepairCopyState("idle");
  }

  function closePathRepair() {
    setPathRepairTool(null);
    setPathRepairCopyState("idle");
  }

  return (
    <main className="app-shell">
      <header className="hero">
        <div>
          <p className="eyebrow">Windows First · V0.3 Network And Privilege</p>
          <h1>AI Coding 环境助手</h1>
          <p className="hero-copy">
            当前阶段补上安装网络配置与权限提示，不启动真实安装器。
          </p>
        </div>
        <button
          type="button"
          onClick={() => void runDetectAll()}
          disabled={isDetectingAll}
        >
          {isDetectingAll ? "检测中..." : "重新检测全部"}
        </button>
      </header>

      {!adminState.isAdmin && adminState.checked ? (
        <section className="panel">
          <div className="panel-header">
            <h2>权限提示</h2>
            <p>当前不是管理员，安装时可能触发 UAC。</p>
          </div>
          <div className="quick-actions">
            <button type="button" onClick={() => void runRestartAsAdmin()}>
              以管理员身份重启
            </button>
          </div>
        </section>
      ) : null}

      <section className="panel">
        <div className="panel-header">
          <h2>基础环境</h2>
          <p>这一步先接入网络设置与权限提示，真实 Git / Node / Python 安装器后续再接。</p>
        </div>
        <ToolTable
          rows={groupedRows.base}
          onDetect={runDetectOne}
          onInstall={runInstall}
          onReinstall={runReinstall}
          onInstallLatest={runInstallLatest}
          onCancelInstall={runCancelInstall}
          onOpenPathRepair={openPathRepair}
          installState={installState}
        />
      </section>

      <section className="panel">
        <div className="panel-header">
          <h2>AI Coding 工具</h2>
          <p>Claude / Codex / OpenCode 已接入安装，ccSwitch 仍在后续 commit 再补。</p>
        </div>
        <ToolTable
          rows={groupedRows.ai}
          onDetect={runDetectOne}
          onInstall={runInstall}
          onReinstall={runReinstall}
          onInstallLatest={runInstallLatest}
          onCancelInstall={runCancelInstall}
          onOpenPathRepair={openPathRepair}
          installState={installState}
        />
      </section>

      <section className="panel split-panel">
        <div className="panel-header">
          <h2>ccSwitch Path</h2>
          <p>
            Save a manual ccSwitch executable path, then re-detect or open it.
            Detection still only probes paths and never launches the GUI.
          </p>
        </div>
        <div className="network-card">
          <label className="network-stat" htmlFor="ccswitch-path-input">
            <span>ccSwitch executable</span>
            <input
              id="ccswitch-path-input"
              type="text"
              value={ccswitchPathInput}
              onChange={(event) => setCcswitchPathInput(event.target.value)}
              placeholder="C:\\Program Files\\ccswitch\\ccswitch.exe"
            />
          </label>
          <div className="action-group">
            <button type="button" onClick={() => void saveCcSwitchPath()}>
              Save ccSwitch path
            </button>
            <button
              type="button"
              onClick={() => void runOpenCcSwitch()}
              disabled={!ccswitchRow?.executablePath}
              title={
                ccswitchRow?.executablePath
                  ? ccswitchRow.executablePath
                  : "Save a valid ccSwitch path or install it first."
              }
            >
              Open ccSwitch
            </button>
            <button type="button" onClick={() => void runDetectOne("ccswitch")}>
              Re-detect ccSwitch
            </button>
          </div>
          <p className="tool-detail">
            Configured download sources: {config.ccswitchDownloadSources.length}. If all sources
            fail or no source is configured, save a manual ccSwitch path instead.
          </p>
        </div>
      </section>

      <section className="panel split-panel">
        <div className="panel-header">
          <h2>安装网络</h2>
          <p>配置会写入 AppConfig；npm 镜像后续只通过单次命令参数使用。</p>
        </div>
        <div className="network-card">
          <div className="network-stat">
            <span>当前模式</span>
            <strong>{config.installNetwork.mode}</strong>
          </div>
          <div className="network-stat">
            <span>npm registry</span>
            <strong>
              {config.installNetwork.npmRegistry === "custom"
                ? config.installNetwork.customNpmRegistry ?? "custom"
                : config.installNetwork.npmRegistry}
            </strong>
          </div>
          <p className="network-warning">
            注意：winget 不保证读取本客户端的临时代理设置。
          </p>
          <button type="button">打开安装网络设置</button>
        </div>
      </section>

      {pathRepairTool?.executablePath ? (
        <PathRepairModal
          tool={pathRepairTool}
          directory={pathDirectory(pathRepairTool.executablePath)}
          command={pathRepairCommand(pathDirectory(pathRepairTool.executablePath))}
          copyState={pathRepairCopyState}
          onCopy={copyPathRepairCommand}
          onClose={closePathRepair}
        />
      ) : null}
    </main>
  );
}

type ToolTableProps = {
  rows: ToolStatus[];
  onDetect: (toolId: ToolId) => Promise<void>;
  onInstall: (toolId: ToolId) => Promise<void>;
  onReinstall: (toolId: ToolId) => Promise<void>;
  onInstallLatest: (toolId: ToolId) => Promise<void>;
  onCancelInstall: () => Promise<void>;
  onOpenPathRepair: (tool: ToolStatus) => void;
  installState: InstallTaskViewState;
};

function ToolTable({
  rows,
  onDetect,
  onInstall,
  onReinstall,
  onInstallLatest,
  onCancelInstall,
  onOpenPathRepair,
  installState,
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
            const meta = statusMeta[row.status];
            const canInstall = INSTALLABLE_TOOL_IDS.includes(row.id);
            const canInstallLatest = LATEST_INSTALLABLE_TOOL_IDS.includes(row.id);
            const showInstallButton = canInstall && row.status === "missing";
            const showReinstallButton =
              canInstall &&
              row.status !== "missing" &&
              row.status !== "checking" &&
              !(
                installState.isInstalling && installState.activeToolId === row.id
              );
            const showInstallLatestButton =
              canInstallLatest &&
              row.status !== "checking" &&
              !(
                installState.isInstalling && installState.activeToolId === row.id
              );
            const showCancelButton =
              installState.isInstalling && installState.activeToolId === row.id;
            const showPathRepairButton =
              row.status === "installed_but_path_missing" && Boolean(row.executablePath);
            const disableInstallButton =
              installState.isInstalling &&
              installState.activeToolId !== row.id;

            return (
              <tr key={row.id}>
                <td>
                  <div className="tool-name">{row.name}</div>
                  {row.suggestion ? (
                    <div className="tool-detail">{row.suggestion}</div>
                  ) : null}
                </td>
                <td>
                  <span className={`status-badge ${meta.className}`}>{meta.label}</span>
                </td>
                <td className="mono">{row.version ?? "-"}</td>
                <td>
                  <div className="path-cell" title={row.executablePath ?? "-"}>
                    {row.executablePath ?? "-"}
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
                          disabled={disableInstallButton}
                          onClick={() => void onInstall(row.id)}
                        >
                          安装
                        </button>
                      ) : null}
                      {showReinstallButton ? (
                        <button
                          type="button"
                          className="ghost-button"
                          disabled={disableInstallButton}
                          onClick={() => void onReinstall(row.id)}
                        >
                          Reinstall
                        </button>
                      ) : null}
                      {showInstallLatestButton ? (
                        <button
                          type="button"
                          className="ghost-button"
                          disabled={disableInstallButton}
                          onClick={() => void onInstallLatest(row.id)}
                        >
                          Install latest
                        </button>
                      ) : null}
                      {showCancelButton ? (
                        <button
                          type="button"
                          className="ghost-button"
                          onClick={() => void onCancelInstall()}
                        >
                          取消安装
                        </button>
                      ) : null}
                      {showPathRepairButton ? (
                        <button
                          type="button"
                          className="ghost-button"
                          onClick={() => onOpenPathRepair(row)}
                        >
                          PATH repair instructions
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
                  {row.errorMessage ? (
                    <div className="row-message">{row.errorMessage}</div>
                  ) : null}
                </td>
              </tr>
            );
          })}
        </tbody>
      </table>
    </div>
  );
}

type PathRepairModalProps = {
  tool: ToolStatus;
  directory: string;
  command: string;
  copyState: "idle" | "copied" | "failed";
  onCopy: (command: string) => Promise<void>;
  onClose: () => void;
};

function PathRepairModal({
  tool,
  directory,
  command,
  copyState,
  onCopy,
  onClose,
}: PathRepairModalProps) {
  return (
    <div className="modal-backdrop" role="presentation">
      <section
        className="path-repair-modal"
        role="dialog"
        aria-modal="true"
        aria-labelledby="path-repair-title"
      >
        <div className="modal-header">
          <div>
            <p className="eyebrow">Manual PATH Repair</p>
            <h2 id="path-repair-title">PATH repair instructions</h2>
          </div>
          <button type="button" className="ghost-button" onClick={onClose}>
            Close
          </button>
        </div>

        <div className="repair-grid">
          <div className="network-stat">
            <span>Tool</span>
            <strong>{tool.name}</strong>
          </div>
          <div className="network-stat">
            <span>Executable path</span>
            <strong className="mono breakable">{tool.executablePath}</strong>
          </div>
          <div className="network-stat">
            <span>Directory to add to PATH</span>
            <strong className="mono breakable">{directory}</strong>
          </div>
        </div>

        <p className="network-warning">
          This client will not automatically modify PATH. Copy the command below and run it
          manually in PowerShell if you want to update your user PATH.
        </p>

        <ol className="manual-steps">
          <li>Review the executable path and directory above.</li>
          <li>Copy the user-level PATH command.</li>
          <li>Run it manually in PowerShell, then restart terminals so PATH refreshes.</li>
          <li>Come back and re-detect the tool.</li>
        </ol>

        <pre className="command-preview">
          <code>{command}</code>
        </pre>

        <div className="action-group">
          <button type="button" onClick={() => void onCopy(command)}>
            Copy manual PATH command
          </button>
          {copyState === "copied" ? (
            <span className="copy-state">Copied to clipboard.</span>
          ) : null}
          {copyState === "failed" ? (
            <span className="copy-state error">Clipboard copy failed.</span>
          ) : null}
        </div>
      </section>
    </div>
  );
}

function pathDirectory(executablePath: string): string {
  const slashIndex = Math.max(
    executablePath.lastIndexOf("\\"),
    executablePath.lastIndexOf("/"),
  );
  return slashIndex >= 0 ? executablePath.slice(0, slashIndex) : executablePath;
}

function pathRepairCommand(directory: string): string {
  const escapedDirectory = directory.replace(/'/g, "''");
  return [
    `$dir = '${escapedDirectory}'`,
    `$userPath = [Environment]::GetEnvironmentVariable("Path", "User")`,
    `$parts = @($userPath -split ';' | Where-Object { $_ })`,
    `if ($parts -notcontains $dir) {`,
    `  $next = (@($parts) + $dir) -join ';'`,
    `  [Environment]::SetEnvironmentVariable("Path", $next, "User")`,
    `}`,
  ].join("\n");
}

export default App;
