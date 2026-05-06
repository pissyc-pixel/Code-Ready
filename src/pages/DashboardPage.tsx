import AdminStatePanel from "../components/status/AdminStatePanel";
import InstallActivityPanel from "../components/status/InstallActivityPanel";
import Card from "../components/ui/Card";
import StatusBadge from "../components/ui/StatusBadge";
import type { AppConfig } from "../types/config";
import type { InstallTaskViewState } from "../types/install";
import type { ToolId, ToolStatus } from "../types/tool";

type DashboardPageProps = {
  rows: ToolStatus[];
  groupedRows: {
    base: ToolStatus[];
    ai: ToolStatus[];
  };
  config: AppConfig;
  installState: InstallTaskViewState;
  activeInstallTool?: ToolStatus;
  latestLogLine?: string;
  logLines: string[];
  adminChecked: boolean;
  isAdmin: boolean;
  quickActionMessage: string;
  logActionMessage: string;
  canOpenCcSwitch: boolean;
  onDetectAll: () => Promise<void>;
  onDetect: (toolId: ToolId) => Promise<void>;
  onInstall: (toolId: ToolId) => Promise<void>;
  onReinstall: (toolId: ToolId) => Promise<void>;
  onCancelInstall: () => Promise<void>;
  onRestartAsAdmin: () => Promise<void>;
  onOpenCcSwitch: () => Promise<void>;
  onOpenSubscriptionPage: () => Promise<void>;
  onExportDiagnostics: () => Promise<void>;
  onNavigate: (pageId: string) => void;
};

type DashboardAction = {
  id: string;
  title: string;
  description: string;
  actionLabel: string;
  action: () => void;
};

const ATTENTION_STATUSES = new Set([
  "broken",
  "detect_failed",
  "install_failed",
  "installed_but_path_missing",
]);

function DashboardPage({
  rows,
  groupedRows,
  config,
  installState,
  activeInstallTool,
  latestLogLine,
  logLines,
  adminChecked,
  isAdmin,
  quickActionMessage,
  logActionMessage,
  canOpenCcSwitch,
  onDetectAll,
  onDetect,
  onInstall,
  onReinstall,
  onCancelInstall,
  onRestartAsAdmin,
  onOpenCcSwitch,
  onOpenSubscriptionPage,
  onExportDiagnostics,
  onNavigate,
}: DashboardPageProps) {
  const readyCount = rows.filter((row) => row.status === "installed").length;
  const attentionCount = rows.filter((row) => ATTENTION_STATUSES.has(row.status)).length;
  const missingCount = rows.filter((row) => row.status === "missing").length;
  const checkingCount = rows.filter((row) => row.status === "checking").length;
  const latestLogs = logLines.slice(-6).reverse();
  const nextActions = buildNextActions({
    rows,
    config,
    onDetect,
    onInstall,
    onReinstall,
    onNavigate,
  });
  const npmRegistryLabel =
    config.installNetwork.npmRegistry === "custom"
      ? config.installNetwork.customNpmRegistry ?? "custom"
      : config.installNetwork.npmRegistry;

  return (
    <div className="dashboard-layout">
      <Card
        className="dashboard-hero"
        eyebrow="Dashboard"
        title="一眼看懂当前环境状态，按优先级给出下一步动作。"
        description="不会自动执行安装；所有按钮仍然连接到现有检测、安装、日志与配置逻辑。"
        actions={
          <div className="hero-actions">
            <button type="button" onClick={() => void onExportDiagnostics()}>
              导出诊断 zip
            </button>
            <button type="button" onClick={() => void onDetectAll()}>
              重新检测全部
            </button>
          </div>
        }
      >
        <div className="dashboard-stats">
          <StatTile label="已就绪" value={readyCount} tone="success" />
          <StatTile label="需关注" value={attentionCount} tone="warning" />
          <StatTile label="待安装" value={missingCount} tone="info" />
          <StatTile label="检测中" value={checkingCount} tone="muted" />
        </div>

        <div className="dashboard-summary-grid">
          <div className="dashboard-summary-card">
            <span className="summary-kicker">环境状态</span>
            <strong>
              基础环境 {groupedRows.base.length} 项 · AI 工具 {groupedRows.ai.length} 项
            </strong>
            <p>
              安装网络模式为 <code>{config.installNetwork.mode}</code>，npm registry 当前为{" "}
              <code>{npmRegistryLabel}</code>。
            </p>
          </div>
          <div className="dashboard-summary-card">
            <span className="summary-kicker">快捷入口</span>
            <div className="dashboard-inline-actions">
              <button
                type="button"
                className="ghost-button"
                disabled={!canOpenCcSwitch}
                onClick={() => void onOpenCcSwitch()}
              >
                打开 ccSwitch
              </button>
              <button
                type="button"
                className="ghost-button"
                disabled={!config.subscriptionPageUrl}
                onClick={() => void onOpenSubscriptionPage()}
              >
                打开订阅页
              </button>
              <button type="button" className="ghost-button" onClick={() => onNavigate("logs")}>
                查看日志
              </button>
            </div>
            {quickActionMessage ? <p className="inline-message">{quickActionMessage}</p> : null}
            {logActionMessage ? <p className="inline-message">{logActionMessage}</p> : null}
          </div>
        </div>
      </Card>

      <div className="status-panel-stack">
        <AdminStatePanel
          adminChecked={adminChecked}
          isAdmin={isAdmin}
          onRestartAsAdmin={onRestartAsAdmin}
        />

        {installState.isInstalling && activeInstallTool ? (
          <InstallActivityPanel
            toolName={activeInstallTool.name}
            latestLogLine={latestLogLine}
            onCancelInstall={onCancelInstall}
            onNavigateLogs={() => onNavigate("logs")}
          />
        ) : null}
      </div>

      <div className="dashboard-grid">
        <Card
          title="下一步"
          description="按优先级排列。每条都对应一个明确动作，不会自动执行。"
        >
          <div className="action-list">
            {nextActions.length > 0 ? (
              nextActions.map((item) => (
                <div key={item.id} className="action-list-item">
                  <div>
                    <strong>{item.title}</strong>
                    <p>{item.description}</p>
                  </div>
                  <button type="button" className="ghost-button" onClick={item.action}>
                    {item.actionLabel}
                  </button>
                </div>
              ))
            ) : (
              <div className="action-list-item">
                <div>
                  <strong>当前没有阻塞项</strong>
                  <p>继续监控日志，或前往 AI 工具页逐项确认版本与路径。</p>
                </div>
                <button
                  type="button"
                  className="ghost-button"
                  onClick={() => onNavigate("aiTools")}
                >
                  前往 AI 工具页
                </button>
              </div>
            )}
          </div>
        </Card>

        <Card title="基础环境" description="Git / Node / npm / Python 的真实检测结果。">
          <div className="dashboard-tool-list">
            {groupedRows.base.map((row) => (
              <ToolOverviewRow key={row.id} row={row} />
            ))}
          </div>
          <div className="card-footer-actions">
            <button
              type="button"
              className="ghost-button"
              onClick={() => onNavigate("environment")}
            >
              前往基础环境页
            </button>
          </div>
        </Card>

        <Card title="AI 工具" description="Claude / Codex / OpenCode / ccSwitch 的真实检测结果。">
          <div className="dashboard-tool-list">
            {groupedRows.ai.map((row) => (
              <ToolOverviewRow key={row.id} row={row} />
            ))}
          </div>
          <div className="card-footer-actions">
            <button type="button" className="ghost-button" onClick={() => onNavigate("aiTools")}>
              前往 AI 工具页
            </button>
          </div>
        </Card>

        <Card title="最近日志" description="实时安装输出与诊断信息仍然来自原有日志状态。">
          <div className="dashboard-log">
            {latestLogs.length > 0 ? (
              latestLogs.map((line, index) => (
                <div className="dashboard-log-line" key={`${index}-${line}`}>
                  {line}
                </div>
              ))
            ) : (
              <div className="dashboard-log-empty">暂无日志输出。</div>
            )}
          </div>
        </Card>

        <Card title="安全边界" description="这些内容目前是静态说明，后续可以再按设计稿细化。">
          <ul className="boundary-list">
            <li>不保存 API Key，也不读取明文。</li>
            <li>不自动写 Provider 配置。</li>
            <li>不接管系统代理。</li>
            <li>不自动修改 PATH，只提供修复说明。</li>
            <li>不做一键全量安装。</li>
          </ul>
        </Card>
      </div>
    </div>
  );
}

type BuildNextActionsArgs = {
  rows: ToolStatus[];
  config: AppConfig;
  onDetect: (toolId: ToolId) => Promise<void>;
  onInstall: (toolId: ToolId) => Promise<void>;
  onReinstall: (toolId: ToolId) => Promise<void>;
  onNavigate: (pageId: string) => void;
};

function buildNextActions({
  rows,
  config,
  onDetect,
  onInstall,
  onReinstall,
  onNavigate,
}: BuildNextActionsArgs): DashboardAction[] {
  const actions: DashboardAction[] = [];

  for (const row of rows) {
    if (row.status === "missing") {
      actions.push({
        id: `${row.id}-install`,
        title: `安装 ${row.name}`,
        description: "工具缺失，建议先补齐安装。",
        actionLabel: "立即安装",
        action: () => void onInstall(row.id),
      });
      continue;
    }

    if (row.status === "broken") {
      actions.push({
        id: `${row.id}-reinstall`,
        title: `重装 ${row.name}`,
        description: row.errorMessage ?? "版本探测或执行异常，建议重装。",
        actionLabel: "重新安装",
        action: () => void onReinstall(row.id),
      });
      continue;
    }

    if (row.status === "detect_failed" || row.status === "install_failed") {
      actions.push({
        id: `${row.id}-detect`,
        title: `重新检测 ${row.name}`,
        description: row.errorMessage ?? "最近一次检测失败，请重新确认。",
        actionLabel: "重新检测",
        action: () => void onDetect(row.id),
      });
      continue;
    }

    if (row.status === "installed_but_path_missing") {
      actions.push({
        id: `${row.id}-path`,
        title: `处理 ${row.name} 的 PATH 缺失`,
        description: row.errorMessage ?? "工具已安装，但当前终端 PATH 尚未刷新。",
        actionLabel: "查看基础环境页",
        action: () => onNavigate("environment"),
      });
    }
  }

  if (!config.subscriptionPageUrl) {
    actions.push({
      id: "subscription-url",
      title: "补充订阅页入口",
      description: "当前还没有配置订阅页 URL，后续将无法一键打开。",
      actionLabel: "前往 ccSwitch 页",
      action: () => onNavigate("ccswitch"),
    });
  }

  return actions.slice(0, 5);
}

type StatTileProps = {
  label: string;
  value: number;
  tone: "success" | "warning" | "info" | "muted";
};

function StatTile({ label, value, tone }: StatTileProps) {
  return (
    <div className={`stat-tile stat-${tone}`}>
      <span>{label}</span>
      <strong>{value}</strong>
    </div>
  );
}

function ToolOverviewRow({ row }: { row: ToolStatus }) {
  return (
    <div className="tool-overview-row">
      <div className="tool-overview-heading">
        <strong>{row.name}</strong>
        <StatusBadge status={row.status} />
      </div>
      <p>{row.executablePath ?? row.version ?? "等待检测结果"}</p>
      {row.errorMessage ? <p className="tool-overview-error">{row.errorMessage}</p> : null}
    </div>
  );
}

export default DashboardPage;
