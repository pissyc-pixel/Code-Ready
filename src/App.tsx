import "./App.css";

type ToolInstallStatus =
  | "checking"
  | "installed"
  | "missing"
  | "installing"
  | "install_failed"
  | "detect_failed"
  | "installed_but_path_missing"
  | "broken";

type ToolRow = {
  name: string;
  status: ToolInstallStatus;
  version: string;
  path: string;
  actions: string[];
  message?: string;
  detail?: string;
};

const baseTools: ToolRow[] = [
  {
    name: "winget",
    status: "installed",
    version: "v1.28.240",
    path: "C:\\Users\\Administrator\\AppData\\Local\\Microsoft\\WindowsApps\\winget.exe",
    actions: ["重新检测", "查看日志"],
  },
  {
    name: "Git / Git Bash",
    status: "missing",
    version: "—",
    path: "—",
    actions: ["安装 Git for Windows", "重新检测", "查看日志"],
    detail: "Git CLI 与 Git Bash 会在后续版本显示子状态。",
  },
  {
    name: "Node.js",
    status: "installed",
    version: "v24.x",
    path: "D:\\nodejs\\node.exe",
    actions: ["重新检测", "重新安装", "打开所在位置", "查看日志"],
  },
  {
    name: "npm",
    status: "installed_but_path_missing",
    version: "v11.x",
    path: "%APPDATA%\\npm\\npm.cmd",
    actions: ["查看修复说明", "复制 PATH 修复命令", "重新检测", "查看日志"],
    message: "检测到可执行文件，但终端 PATH 可能未刷新。",
  },
  {
    name: "Python 3.11",
    status: "detect_failed",
    version: "—",
    path: "—",
    actions: ["重新检测", "查看日志", "复制错误"],
    message: "检测命令超时，后续会显示更具体的异常摘要。",
  },
];

const aiTools: ToolRow[] = [
  {
    name: "Claude Code",
    status: "missing",
    version: "—",
    path: "—",
    actions: ["安装", "重新检测", "查看日志"],
  },
  {
    name: "Codex CLI",
    status: "checking",
    version: "—",
    path: "—",
    actions: [],
    message: "正在等待后端检测结果事件。",
  },
  {
    name: "OpenCode",
    status: "broken",
    version: "v0.9.x",
    path: "C:\\Users\\Administrator\\AppData\\Roaming\\npm\\opencode.cmd",
    actions: ["重新安装", "重新检测", "查看日志", "复制错误"],
    message: "命令存在，但执行版本探测失败。",
  },
  {
    name: "ccSwitch",
    status: "installing",
    version: "v0.x",
    path: "%APPDATA%\\ai-coding-installer\\ccswitch\\ccswitch.exe",
    actions: ["查看日志", "取消安装"],
    message: "安装进度会在 V0.3 接入 install:progress。",
  },
  {
    name: "Gemini CLI 示例占位",
    status: "install_failed",
    version: "—",
    path: "—",
    actions: ["重试安装", "重新检测", "查看日志", "复制错误"],
    message: "这行仅用于在 V0.1 覆盖 install_failed 状态，不代表本轮实现范围。",
  },
];

const statusMeta: Record<
  ToolInstallStatus,
  { label: string; className: string }
> = {
  checking: { label: "检测中", className: "checking" },
  installed: { label: "已安装", className: "installed" },
  missing: { label: "未安装", className: "missing" },
  installing: { label: "安装中", className: "installing" },
  install_failed: { label: "安装失败", className: "install-failed" },
  detect_failed: { label: "检测失败", className: "detect-failed" },
  installed_but_path_missing: {
    label: "PATH 缺失",
    className: "path-missing",
  },
  broken: { label: "已损坏", className: "broken" },
};

function App() {
  return (
    <main className="app-shell">
      <header className="hero">
        <div>
          <p className="eyebrow">Windows First · V0.1 Mock Dashboard</p>
          <h1>AI Coding 环境助手</h1>
          <p className="hero-copy">
            先把状态看清楚，再逐步接入真实检测、日志脱敏和后台安装任务。
          </p>
        </div>
        <button type="button">重新检测全部</button>
      </header>

      <section className="panel">
        <div className="panel-header">
          <h2>基础环境</h2>
          <p>第一版只做 Windows，且不提供一键全装。</p>
        </div>
        <ToolTable rows={baseTools} />
      </section>

      <section className="panel">
        <div className="panel-header">
          <h2>AI Coding 工具</h2>
          <p>安装、重装、修复入口后续会逐步接到 Tauri command 和事件。</p>
        </div>
        <ToolTable rows={aiTools} />
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

function ToolTable({ rows }: { rows: ToolRow[] }) {
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
              <tr key={row.name}>
                <td>
                  <div className="tool-name">{row.name}</div>
                  {row.detail ? <div className="tool-detail">{row.detail}</div> : null}
                </td>
                <td>
                  <span className={`status-badge ${meta.className}`}>{meta.label}</span>
                </td>
                <td className="mono">{row.version}</td>
                <td>
                  <div className="path-cell" title={row.path}>
                    {row.path}
                  </div>
                </td>
                <td>
                  {row.status === "checking" ? (
                    <span className="checking-text">等待结果...</span>
                  ) : (
                    <div className="action-group">
                      {row.actions.map((action) => (
                        <button key={action} type="button" className="ghost-button">
                          {action}
                        </button>
                      ))}
                    </div>
                  )}
                  {row.message ? <div className="row-message">{row.message}</div> : null}
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
