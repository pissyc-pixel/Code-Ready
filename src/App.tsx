import { useEffect, useMemo, useState } from "react";
import "./App.css";
import { mockAiTools, mockBaseTools } from "./data/mockTools";
import { useDetectEvents } from "./hooks/useDetectEvents";
import { useInstallEvents } from "./hooks/useInstallEvents";
import {
  cancelInstall,
  detectAllTools,
  detectTool,
  installTool,
} from "./lib/api";
import { statusMeta } from "./lib/toolStatus";
import type {
  InstallStatusEvent,
  InstallTaskViewState,
} from "./types/install";
import type { ToolId, ToolStatus } from "./types/tool";

const initialRows = [...mockBaseTools, ...mockAiTools];
const INSTALLABLE_TOOL_IDS: ToolId[] = ["git", "node", "python"];

function App() {
  const [rows, setRows] = useState<ToolStatus[]>(() =>
    initialRows.map((tool) => ({ ...tool, status: "checking" })),
  );
  const [isDetectingAll, setIsDetectingAll] = useState(false);
  const [installState, setInstallState] = useState<InstallTaskViewState>({
    isInstalling: false,
  });

  useDetectEvents((event) => {
    applyResult(event.result);
  });

  useInstallEvents({
    onStatus: applyInstallStatus,
  });

  useEffect(() => {
    void runDetectAll();
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
    await installTool(toolId);
  }

  async function runCancelInstall() {
    await cancelInstall();
  }

  return (
    <main className="app-shell">
      <header className="hero">
        <div>
          <p className="eyebrow">Windows First · V0.3 Install Event Skeleton</p>
          <h1>AI Coding 环境助手</h1>
          <p className="hero-copy">
            这一步只接入安装状态、事件和互斥按钮，不启动真实安装器。
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

      <section className="panel">
        <div className="panel-header">
          <h2>基础环境</h2>
          <p>第一步只接线安装事件骨架，Git / Node / Python 后续再接真实安装器。</p>
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
          <p>当前阶段仍只做检测，不接入 Claude / Codex / OpenCode / ccSwitch 安装。</p>
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
          <p>网络设置入口保留，完整 AppConfig 和代理策略会在下一个 commit 接入。</p>
        </div>
        <div className="network-card">
          <div className="network-stat">
            <span>当前模式</span>
            <strong>不使用代理</strong>
          </div>
          <div className="network-stat">
            <span>npm registry</span>
            <strong>default</strong>
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
