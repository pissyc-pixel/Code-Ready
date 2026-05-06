import Card from "../components/ui/Card";
import ToolTable from "../components/tools/ToolTable";
import type { InstallTaskViewState } from "../types/install";
import type { ToolId, ToolStatus } from "../types/tool";

const ENV_TOOL_SUBTITLES: Partial<Record<ToolId, string>> = {
  winget: "Windows package manager",
  git: "Source control",
  node: "JavaScript runtime",
  npm: "Package manager",
  python: "Runtime",
};

type EnvPageProps = {
  rows: ToolStatus[];
  isDetectingAll: boolean;
  installState: InstallTaskViewState;
  onDetectAll: () => Promise<void>;
  onDetect: (toolId: ToolId) => Promise<void>;
  onInstall: (toolId: ToolId) => Promise<void>;
  onReinstall: (toolId: ToolId) => Promise<void>;
  onInstallLatest: (toolId: ToolId) => Promise<void>;
  onCancelInstall: () => Promise<void>;
  onOpenPathRepair: (tool: ToolStatus) => void;
  onNavigateLogs: () => void;
};

function EnvPage({
  rows,
  isDetectingAll,
  installState,
  onDetectAll,
  onDetect,
  onInstall,
  onReinstall,
  onInstallLatest,
  onCancelInstall,
  onOpenPathRepair,
  onNavigateLogs,
}: EnvPageProps) {
  return (
    <div className="tool-page-layout">
      <section className="page-section panel" role="region" aria-label="基础环境">
        <div className="panel-header">
          <div>
            <h2>基础环境</h2>
            <p>
              Git / Node / npm / Python 是 AI Coding CLI 的前置依赖。每项独立检测、
              独立安装。
            </p>
          </div>
          <div className="action-group">
            <button type="button" className="ghost-button" onClick={onNavigateLogs}>
              查看日志
            </button>
            <button type="button" onClick={() => void onDetectAll()} disabled={isDetectingAll}>
              {isDetectingAll ? "检测中..." : "重新检测"}
            </button>
          </div>
        </div>

        <ToolTable
          rows={rows}
          installState={installState}
          subtitles={ENV_TOOL_SUBTITLES}
          getRowNote={buildEnvironmentRowNote}
          onDetect={onDetect}
          onInstall={onInstall}
          onReinstall={onReinstall}
          onInstallLatest={onInstallLatest}
          onCancelInstall={onCancelInstall}
          onOpenPathRepair={onOpenPathRepair}
        />
      </section>

      <div className="tool-page-sidebar">
        <Card
          title="检测说明"
          description="这些信息只读。如果你需要进一步排查，请前往日志页。"
        >
          <div className="tool-info-grid">
            <div className="tool-info-item">
              <span>检测方式</span>
              <strong>which → --version → 注册表 / npm 全局目录</strong>
            </div>
            <div className="tool-info-item">
              <span>超时</span>
              <strong>单工具 5 秒</strong>
            </div>
            <div className="tool-info-item">
              <span>最近一次</span>
              <strong>{formatLastChecked(rows)}</strong>
            </div>
          </div>
        </Card>

        <Card
          title="状态说明"
          description="installed、missing、broken、installing、detect_failed、PATH 缺失 全部沿用真实事件状态。"
        >
          <ul className="boundary-list">
            <li>PATH 缺失不会自动修改环境变量，只提供修复说明。</li>
            <li>检测失败时优先展示后端返回的真实错误消息。</li>
            <li>安装中的工具可以单独取消，不影响其他工具状态。</li>
          </ul>
        </Card>
      </div>
    </div>
  );
}

function buildEnvironmentRowNote(row: ToolStatus): string | undefined {
  if (row.errorMessage) {
    return row.errorMessage;
  }

  if (row.id === "npm" && row.status === "installed_but_path_missing") {
    return "终端 PATH 未刷新。重启终端，或查看 PATH 修复说明。";
  }

  return row.suggestion;
}

function formatLastChecked(rows: ToolStatus[]): string {
  const latest = rows
    .map((row) => Date.parse(row.lastCheckedAt))
    .filter((value) => Number.isFinite(value))
    .sort((left, right) => right - left)[0];

  if (!latest) {
    return "暂无";
  }

  return new Intl.DateTimeFormat("zh-CN", {
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
    hour12: false,
  }).format(new Date(latest));
}

export default EnvPage;
