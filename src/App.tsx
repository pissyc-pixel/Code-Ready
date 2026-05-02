import { useEffect, useMemo, useState } from "react";
import "./App.css";
import { mockAiTools, mockBaseTools } from "./data/mockTools";
import { useDetectEvents } from "./hooks/useDetectEvents";
import { detectAllTools, detectTool } from "./lib/api";
import { statusMeta } from "./lib/toolStatus";
import type { ToolId, ToolStatus } from "./types/tool";

const initialRows = [...mockBaseTools, ...mockAiTools];

function App() {
  const [rows, setRows] = useState<ToolStatus[]>(() =>
    initialRows.map((tool) => ({ ...tool, status: "checking" })),
  );
  const [isDetectingAll, setIsDetectingAll] = useState(false);

  useDetectEvents((event) => {
    applyResult(event.result);
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

  return (
    <main className="app-shell">
      <header className="hero">
        <div>
          <p className="eyebrow">Windows First · V0.2 Detection Dashboard</p>
          <h1>AI Coding 环境助手</h1>
          <p className="hero-copy">
            当前阶段只做真实检测闭环，不做安装、不接管代理、不改系统 PATH。
          </p>
        </div>
        <button type="button" onClick={() => void runDetectAll()} disabled={isDetectingAll}>
          {isDetectingAll ? "检测中..." : "重新检测全部"}
        </button>
      </header>

      <section className="panel">
        <div className="panel-header">
          <h2>基础环境</h2>
          <p>V0.2 将通过 Tauri invoke + Rust detector 返回真实状态。</p>
        </div>
        <ToolTable rows={groupedRows.base} onDetect={runDetectOne} />
      </section>

      <section className="panel">
        <div className="panel-header">
          <h2>AI Coding 工具</h2>
          <p>Claude Code / Codex / OpenCode / ccSwitch 先做检测，不做安装。</p>
        </div>
        <ToolTable rows={groupedRows.ai} onDetect={runDetectOne} />
      </section>

      <section className="panel split-panel">
        <div className="panel-header">
          <h2>安装网络</h2>
          <p>网络设置入口保留，但本阶段不触发任何安装任务。</p>
        </div>
        <div className="network-card">
          <div className="network-stat">
            <span>当前模式</span>
            <strong>不使用代理</strong>
          </div>
          <div className="network-stat">
            <span>npm registry</span>
            <strong>npmmirror</strong>
          </div>
          <p className="network-warning">
            注意：即使后续进入安装阶段，winget 也不保证使用 HTTP_PROXY /
            HTTPS_PROXY / ALL_PROXY。
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
};

function ToolTable({ rows, onDetect }: ToolTableProps) {
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
