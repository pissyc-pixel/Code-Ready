import { useEffect, useMemo, useState } from "react";
import "./App.css";
import { mockAiTools, mockBaseTools } from "./data/mockTools";
import { useDetectEvents } from "./hooks/useDetectEvents";
import { useInstallEvents } from "./hooks/useInstallEvents";
import {
  cancelInstall,
  detectAllTools,
  detectTool,
  exportDiagnosticsLogZip,
  getConfig,
  getLogPreview,
  installLatestTool,
  installTool,
  isAdmin,
  openCcSwitch,
  openFullLogFile,
  openLogDirectory,
  openSubscriptionPage,
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
const LOG_VIEWER_LINE_LIMIT = 5000;

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
  const [subscriptionPageUrlInput, setSubscriptionPageUrlInput] = useState("");
  const [quickActionMessage, setQuickActionMessage] = useState("");
  const [logLines, setLogLines] = useState<string[]>([]);
  const [logActionMessage, setLogActionMessage] = useState("");
  const [isExportingDiagnostics, setIsExportingDiagnostics] = useState(false);
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
    onProgress: applyLogProgress,
    onStatus: applyInstallStatus,
  });

  useEffect(() => {
    void runDetectAll();
    void loadConfig();
    void loadPrivilege();
    void loadLogPreview();
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

  function applyLogProgress(event: {
    stream: string;
    line: string;
    timestamp: string;
  }) {
    appendLogLine(`[${event.timestamp}] [${event.stream}] ${event.line}`);
  }

  function appendLogLine(line: string) {
    setLogLines((currentLines) =>
      [...currentLines, line].slice(-LOG_VIEWER_LINE_LIMIT),
    );
  }

  async function loadConfig() {
    const nextConfig = await getConfig();
    setConfig(nextConfig);
    setCcswitchPathInput(nextConfig.ccswitchPath ?? "");
    setSubscriptionPageUrlInput(nextConfig.subscriptionPageUrl ?? "");
  }

  async function loadPrivilege() {
    const nextIsAdmin = await isAdmin();
    setAdminState({
      checked: true,
      isAdmin: nextIsAdmin,
    });
  }

  async function loadLogPreview() {
    try {
      const preview = await getLogPreview(LOG_VIEWER_LINE_LIMIT);
      setLogLines(preview.slice(-LOG_VIEWER_LINE_LIMIT));
      setLogActionMessage("");
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      setLogActionMessage(`Log preview unavailable: ${message}`);
    }
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

  async function saveSubscriptionPageUrl() {
    const nextConfig = await updateConfig({
      subscriptionPageUrl: subscriptionPageUrlInput,
    });
    setConfig(nextConfig);
    setSubscriptionPageUrlInput(nextConfig.subscriptionPageUrl ?? "");
    setQuickActionMessage("节点订阅网页已保存。");
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

  async function runOpenSubscriptionPage() {
    try {
      await openSubscriptionPage();
      setQuickActionMessage("已用默认浏览器打开节点订阅网页。");
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      setQuickActionMessage(message);
    }
  }

  async function runOpenFullLogFile() {
    try {
      await openFullLogFile();
      setLogActionMessage("Full log file opened.");
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      setLogActionMessage(`Unable to open full log file: ${message}`);
    }
  }

  async function runOpenLogDirectory() {
    try {
      await openLogDirectory();
      setLogActionMessage("Log directory opened.");
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      setLogActionMessage(`Unable to open log directory: ${message}`);
    }
  }

  async function runExportDiagnosticsLogZip() {
    setIsExportingDiagnostics(true);
    setLogActionMessage("Exporting diagnostics zip...");
    try {
      const result = await exportDiagnosticsLogZip();
      setLogActionMessage(`Diagnostics zip exported: ${result.path}`);
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      setLogActionMessage(`Unable to export diagnostics zip: ${message}`);
    } finally {
      setIsExportingDiagnostics(false);
    }
  }

  function scrollToPanel(id: string) {
    const target = document.getElementById(id);
    if (typeof target?.scrollIntoView === "function") {
      target.scrollIntoView({ behavior: "smooth", block: "start" });
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
          <p className="eyebrow">Windows First - V0.5 Subscription Entry</p>
          <h1>AI Coding 环境助手</h1>
          <p className="hero-copy">
            当前阶段补上订阅入口、快捷操作和配置保存，不处理节点解析、代理或 Provider。
          </p>
        </div>
        <button
          type="button"
          onClick={() => void runDetectAll()}
          disabled={isDetectingAll}
        >
          {isDetectingAll ? "检测中..." : "刷新检测"}
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
          <h2>Quick Actions</h2>
          <p>Common entries only open or jump; they do not save API keys, edit Provider, or take over proxy.</p>
        </div>
        <div className="quick-actions">
          <button
            type="button"
            onClick={() => void runDetectAll()}
            disabled={isDetectingAll}
          >
            {isDetectingAll ? "检测中..." : "重新检测全部"}
          </button>
          <button
            type="button"
            onClick={() => void runOpenCcSwitch()}
            disabled={!ccswitchRow?.executablePath}
            title={
              ccswitchRow?.executablePath
                ? ccswitchRow.executablePath
                : "请先安装或指定 ccSwitch 路径"
            }
          >
            打开 ccSwitch
          </button>
          <button
            type="button"
            onClick={() => void runOpenSubscriptionPage()}
            disabled={!config.subscriptionPageUrl}
            title={
              config.subscriptionPageUrl
                ? config.subscriptionPageUrl
                : "请先配置节点订阅网页"
            }
          >
            打开节点订阅网页
          </button>
          <button type="button" onClick={() => scrollToPanel("install-network-settings")}>
            打开安装网络设置
          </button>
          <button type="button" onClick={() => scrollToPanel("logs-panel")}>
            查看日志
          </button>
        </div>
        {quickActionMessage ? <p className="row-message">{quickActionMessage}</p> : null}
      </section>

      <section className="panel">
        <div className="panel-header">
          <h2>基础环境</h2>
          <p>这一步保留基础依赖检测和安装入口，具体操作仍按单个工具执行。</p>
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
          <p>Claude / Codex / OpenCode / ccSwitch 分别保留检测、安装和启动入口。</p>
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
                : "请先安装或指定 ccSwitch 路径"
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
          <h2>节点订阅网页</h2>
          <p>这里只保存和打开网页入口，不解析订阅、不下载节点、不设置代理。</p>
        </div>
        <div className="network-card">
          <label className="network-stat" htmlFor="subscription-page-url-input">
            <span>节点订阅网页</span>
            <input
              id="subscription-page-url-input"
              type="url"
              value={subscriptionPageUrlInput}
              onChange={(event) => setSubscriptionPageUrlInput(event.target.value)}
              placeholder="https://example.com/dashboard"
            />
          </label>
          <div className="action-group">
            <button type="button" onClick={() => void saveSubscriptionPageUrl()}>
              保存订阅网页
            </button>
            <button type="button" onClick={() => void runOpenSubscriptionPage()}>
              打开订阅网页
            </button>
          </div>
          <p className="tool-detail">
            留空表示不配置；打开时后端会校验必须是 http:// 或 https:// URL。
          </p>
        </div>
      </section>

      <section className="panel split-panel" id="install-network-settings">
        <div className="panel-header">
          <h2>安装网络</h2>
          <p>配置会写入 AppConfig；npm 镜像只通过本客户端发起的安装命令使用。</p>
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
          <button type="button" onClick={() => scrollToPanel("install-network-settings")}>
            当前安装网络设置
          </button>
        </div>
      </section>

      <section className="panel" id="logs-panel">
        <div className="panel-header">
          <h2>Logs</h2>
          <p>Recent install output is capped to the latest 5000 lines in this viewer.</p>
        </div>
        <div className="log-toolbar">
          <div className="log-count">
            Showing {logLines.length} / {LOG_VIEWER_LINE_LIMIT} lines
          </div>
          <div className="action-group">
            <button type="button" onClick={() => void runOpenFullLogFile()}>
              Open full log file
            </button>
            <button type="button" onClick={() => void runOpenLogDirectory()}>
              Open log directory
            </button>
            <button
              type="button"
              onClick={() => void runExportDiagnosticsLogZip()}
              disabled={isExportingDiagnostics}
            >
              {isExportingDiagnostics
                ? "Exporting diagnostics zip..."
                : "Export diagnostics zip"}
            </button>
          </div>
        </div>
        {logActionMessage ? <p className="row-message">{logActionMessage}</p> : null}
        <div className="log-viewer" role="log" aria-label="Install log viewer">
          {logLines.length > 0 ? (
            logLines.map((line, index) => (
              <div className="log-line" key={`${index}-${line}`}>
                {line}
              </div>
            ))
          ) : (
            <div className="log-empty">No log lines yet.</div>
          )}
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
