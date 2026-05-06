import InstallActivityPanel from "../components/status/InstallActivityPanel";
import StatusPanel from "../components/status/StatusPanel";
import ToolTable from "../components/tools/ToolTable";
import Card from "../components/ui/Card";
import type { InstallTaskViewState } from "../types/install";
import type { ToolId, ToolStatus } from "../types/tool";

const ENV_TOOL_SUBTITLES: Partial<Record<ToolId, string>> = {
  winget: "Windows 包管理器",
  git: "版本管理",
  node: "JavaScript 运行时",
  npm: "包管理器",
  python: "Python 运行时",
};

type EnvPageProps = {
  rows: ToolStatus[];
  isDetectingAll: boolean;
  installState: InstallTaskViewState;
  activeInstallTool?: ToolStatus;
  latestLogLine?: string;
  onDetectAll: () => Promise<void>;
  onDetect: (toolId: ToolId) => Promise<void>;
  onInstall: (toolId: ToolId) => Promise<void>;
  onReinstall: (toolId: ToolId) => Promise<void>;
  onInstallLatest: (toolId: ToolId) => Promise<void>;
  onCancelInstall: () => Promise<void>;
  onOpenPathRepair: (tool: ToolStatus) => void;
  onNavigateLogs: () => void;
  onOpenLogDirectory: () => Promise<void>;
};

function EnvPage({
  rows,
  isDetectingAll,
  installState,
  activeInstallTool,
  latestLogLine,
  onDetectAll,
  onDetect,
  onInstall,
  onReinstall,
  onInstallLatest,
  onCancelInstall,
  onOpenPathRepair,
  onNavigateLogs,
  onOpenLogDirectory,
}: EnvPageProps) {
  const pathRows = rows.filter((row) => row.status === "installed_but_path_missing");
  const detectFailedRows = rows.filter((row) => row.status === "detect_failed");
  const installIssueRows = rows.filter(
    (row) => row.status === "install_failed" || row.status === "broken",
  );
  const activeBaseInstall =
    activeInstallTool?.category === "base" ? activeInstallTool : undefined;

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
          onOpenLogDirectory={onOpenLogDirectory}
          onNavigateLogs={onNavigateLogs}
        />
      </section>

      <div className="tool-page-sidebar">
        {activeBaseInstall ? (
          <InstallActivityPanel
            toolName={activeBaseInstall.name}
            latestLogLine={latestLogLine}
            onCancelInstall={onCancelInstall}
            onNavigateLogs={onNavigateLogs}
          />
        ) : null}

        {pathRows.length > 0 ? (
          <StatusPanel
            tone="warning"
            eyebrow="PATH 缺失"
            title="可执行文件存在，但终端 PATH 没刷新"
            description="如果工具是刚刚安装的，先重启终端；如果仍无效，再使用手动 PATH 说明。"
            actions={
              <div className="action-group">
                <button
                  type="button"
                  className="ghost-button"
                  onClick={() => onOpenPathRepair(pathRows[0])}
                >
                  查看 PATH 修复说明
                </button>
                <button type="button" className="ghost-button" onClick={onNavigateLogs}>
                  查看日志
                </button>
              </div>
            }
            items={pathRows.map((row) => ({
              id: row.id,
              title: `${row.name} 已安装，但 PATH 中找不到`,
              description: row.executablePath ?? "未返回可执行文件路径",
              meta: "本客户端不会自动改 PATH，只提供手动修复说明。",
            }))}
          />
        ) : null}

        {detectFailedRows.length > 0 ? (
          <StatusPanel
            tone="danger"
            eyebrow="检测失败"
            title="检测命令超时或返回异常"
            description="优先展示后端返回的真实错误信息，并给出重新检测和日志入口。"
            items={detectFailedRows.map((row) => ({
              id: row.id,
              title: `${row.name} 检测失败`,
              description: row.errorMessage ?? "最近一次检测没有返回可用结果。",
              actions: (
                <div className="action-group">
                  <button type="button" className="ghost-button" onClick={() => void onDetect(row.id)}>
                    重新检测 {row.name}
                  </button>
                  <button
                    type="button"
                    className="ghost-button"
                    onClick={() => void onOpenLogDirectory()}
                  >
                    打开日志目录
                  </button>
                </div>
              ),
            }))}
          />
        ) : null}

        {installIssueRows.length > 0 ? (
          <StatusPanel
            tone="danger"
            eyebrow="安装失败"
            title="安装失败 / 已损坏"
            description="安装失败、版本探测异常或已损坏的工具，会在这里集中给出重试与日志入口。"
            items={installIssueRows.map((row) => ({
              id: row.id,
              title: `${row.name} 需要处理`,
              description: row.errorMessage ?? row.suggestion ?? "建议重新安装并查看日志。",
              actions: (
                <div className="action-group">
                  <button
                    type="button"
                    className="ghost-button"
                    onClick={() => void onReinstall(row.id)}
                  >
                    重新安装 {row.name}
                  </button>
                  <button type="button" className="ghost-button" onClick={onNavigateLogs}>
                    查看日志
                  </button>
                </div>
              ),
            }))}
          />
        ) : null}

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
