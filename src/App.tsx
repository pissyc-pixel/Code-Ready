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
  installTool,
  isAdmin,
  restartAsAdmin,
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
];

function App() {
  const [rows, setRows] = useState<ToolStatus[]>(() =>
    initialRows.map((tool) => ({ ...tool, status: "checking" })),
  );
  const [isDetectingAll, setIsDetectingAll] = useState(false);
  const [installState, setInstallState] = useState<InstallTaskViewState>({
    isInstalling: false,
  });
  const [config, setConfig] = useState<AppConfig>(defaultAppConfig);
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
          onCancelInstall={runCancelInstall}
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
          onCancelInstall={runCancelInstall}
          installState={installState}
        />
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
    </main>
  );
}

type ToolTableProps = {
  rows: ToolStatus[];
  onDetect: (toolId: ToolId) => Promise<void>;
  onInstall: (toolId: ToolId) => Promise<void>;
  onCancelInstall: () => Promise<void>;
  installState: InstallTaskViewState;
};

function ToolTable({
  rows,
  onDetect,
  onInstall,
  onCancelInstall,
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
            const showInstallButton = canInstall && row.status === "missing";
            const showCancelButton =
              installState.isInstalling && installState.activeToolId === row.id;
            const disableInstallButton =
              showInstallButton &&
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
                      {showCancelButton ? (
                        <button
                          type="button"
                          className="ghost-button"
                          onClick={() => void onCancelInstall()}
                        >
                          取消安装
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

export default App;
