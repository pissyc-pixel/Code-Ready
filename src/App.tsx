import "./App.css";
import { mockAiTools, mockBaseTools } from "./data/mockTools";
import { statusMeta } from "./lib/toolStatus";
import type { ToolStatus } from "./types/tool";

type ToolRowProps = {
  rows: ToolStatus[];
};

function App() {
  return (
    <main className="app-shell">
      <header className="hero">
        <div>
          <p className="eyebrow">Windows First · V0.1 Mock Dashboard</p>
          <h1>AI Coding 环境助手</h1>
          <p className="hero-copy">
            先把状态看清楚，再逐步接入真实检测、日志脱敏和后端命令分发。
          </p>
        </div>
        <button type="button">重新检测全部</button>
      </header>

      <section className="panel">
        <div className="panel-header">
          <h2>基础环境</h2>
          <p>第一版只做 Windows，且不提供一键全装。</p>
        </div>
        <ToolTable rows={mockBaseTools} />
      </section>

      <section className="panel">
        <div className="panel-header">
          <h2>AI Coding 工具</h2>
          <p>V0.2 将接入真实检测与事件推送，但当前页仍保持 V0.1 的稳定骨架。</p>
        </div>
        <ToolTable rows={mockAiTools} />
      </section>

      <section className="panel split-panel">
        <div className="panel-header">
          <h2>安装网络</h2>
          <p>临时代理只作用于本客户端发起的安装命令，不接管系统代理。</p>
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
            注意：临时代理不保证影响 winget。如 winget 下载失败，请先开启系统级代理。
          </p>
          <button type="button">打开安装网络设置</button>
        </div>
      </section>

      <section className="panel">
        <div className="panel-header">
          <h2>快捷操作</h2>
          <p>日志查看、订阅网页和 ccSwitch 打开入口会保留在首页。</p>
        </div>
        <div className="quick-actions">
          <button type="button">重新检测全部</button>
          <button type="button" disabled>
            打开 ccSwitch
          </button>
          <button type="button">打开节点订阅网页</button>
          <button type="button">导出诊断日志</button>
        </div>
      </section>
    </main>
  );
}

function ToolTable({ rows }: ToolRowProps) {
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
            const actionText = getActionText(row);

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
                      {actionText.map((action) => (
                        <button key={action} type="button" className="ghost-button">
                          {action}
                        </button>
                      ))}
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

function getActionText(row: ToolStatus): string[] {
  switch (row.status) {
    case "installed":
      return ["重新检测", "打开所在位置", "查看日志"];
    case "missing":
      return row.id === "git"
        ? ["安装 Git for Windows", "重新检测", "查看日志"]
        : ["安装", "重新检测", "查看日志"];
    case "installed_but_path_missing":
      return ["查看修复说明", "复制 PATH 修复命令", "重新检测", "查看日志"];
    case "broken":
      return ["重新安装", "重新检测", "查看日志", "复制错误"];
    case "detect_failed":
      return ["重新检测", "查看日志", "复制错误"];
    case "install_failed":
      return ["重试安装", "重新检测", "查看日志", "复制错误"];
    case "installing":
      return ["查看日志", "取消安装"];
    case "checking":
      return [];
    default:
      return ["重新检测", "查看日志"];
  }
}

export default App;
