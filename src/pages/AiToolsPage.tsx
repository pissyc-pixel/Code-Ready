import InstallActivityPanel from "../components/status/InstallActivityPanel";
import StatusPanel from "../components/status/StatusPanel";
import ToolTable from "../components/tools/ToolTable";
import Card from "../components/ui/Card";
import type { AppConfig } from "../types/config";
import type { InstallTaskViewState } from "../types/install";
import type { ToolId, ToolStatus } from "../types/tool";

const AI_TOOL_SUBTITLES: Partial<Record<ToolId, string>> = {
  claude: "AI CLI",
  codex: "AI CLI",
  opencode: "AI CLI",
  ccswitch: "GUI · 节点切换",
};

const SUPPRESSED_NOTE_PATTERNS = [
  "Detected CLI files, but login or local configuration may still be required.",
  "可能仍需登录",
  "可能仍需本地配置",
  "登录或本地配置",
];

type AiToolsPageProps = {
  rows: ToolStatus[];
  config: AppConfig;
  isDetectingAll: boolean;
  installState: InstallTaskViewState;
  activeInstallTool?: ToolStatus;
  latestLogLine?: string;
  canOpenCcSwitch: boolean;
  onDetectAll: () => Promise<void>;
  onDetect: (toolId: ToolId) => Promise<void>;
  onInstall: (toolId: ToolId) => Promise<void>;
  onReinstall: (toolId: ToolId) => Promise<void>;
  onInstallLatest: (toolId: ToolId) => Promise<void>;
  onCancelInstall: () => Promise<void>;
  onOpenPathRepair: (tool: ToolStatus) => void;
  onOpenLogDirectory: () => Promise<void>;
  onNavigateLogs: () => void;
  onOpenCcSwitch: () => Promise<void>;
  onOpenSubscriptionPage: () => Promise<void>;
};

function AiToolsPage({
  rows,
  config,
  isDetectingAll,
  installState,
  activeInstallTool,
  latestLogLine,
  canOpenCcSwitch,
  onDetectAll,
  onDetect,
  onInstall,
  onReinstall,
  onInstallLatest,
  onCancelInstall,
  onOpenPathRepair,
  onOpenLogDirectory,
  onNavigateLogs,
  onOpenCcSwitch,
  onOpenSubscriptionPage,
}: AiToolsPageProps) {
  const npmRegistryLabel =
    config.installNetwork.npmRegistry === "custom"
      ? config.installNetwork.customNpmRegistry ?? "custom"
      : config.installNetwork.npmRegistry;
  const pathRows = rows.filter((row) => row.status === "installed_but_path_missing");
  const detectFailedRows = rows.filter((row) => row.status === "detect_failed");
  const installIssueRows = rows.filter(
    (row) => row.status === "install_failed" || row.status === "broken",
  );
  const activeAiInstall = activeInstallTool?.category === "ai" ? activeInstallTool : undefined;

  return (
    <div className="tool-page-layout">
      <section className="page-section panel" role="region" aria-label="AI 工具">
        <div className="panel-header">
          <div>
            <h2>AI 工具</h2>
            <p>
              Claude Code / Codex CLI / OpenCode / ccSwitch 的检测、安装、更新与取消逻辑全部保留。
            </p>
          </div>
          <div className="action-group">
            <button
              type="button"
              className="ghost-button"
              onClick={() => void onOpenCcSwitch()}
              disabled={!canOpenCcSwitch}
            >
              打开 ccSwitch
            </button>
            <button
              type="button"
              className="ghost-button"
              onClick={() => void onOpenSubscriptionPage()}
              disabled={!config.subscriptionPageUrl}
            >
              打开订阅页
            </button>
            <button type="button" onClick={() => void onDetectAll()} disabled={isDetectingAll}>
              {isDetectingAll ? "检测中..." : "重新检测"}
            </button>
          </div>
        </div>

        <p className="tool-inline-note">
          每个工具单独安装、单独取消。安装最新版只对支持 `npm -g` 的工具开放。
        </p>

        <ToolTable
          rows={rows}
          installState={installState}
          subtitles={AI_TOOL_SUBTITLES}
          layout="cards"
          getRowNote={buildAiToolRowNote}
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
        {activeAiInstall ? (
          <InstallActivityPanel
            toolName={activeAiInstall.name}
            latestLogLine={latestLogLine}
            onCancelInstall={onCancelInstall}
            onNavigateLogs={onNavigateLogs}
          />
        ) : null}

        {installIssueRows.length > 0 ? (
          <StatusPanel
            tone="danger"
            eyebrow="安装失败"
            title="安装失败 / 已损坏"
            description="保留真实失败原因，并把重试、安装最新版和日志入口集中在一起。"
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
                  {row.id === "claude" || row.id === "codex" || row.id === "opencode" ? (
                    <button
                      type="button"
                      className="ghost-button"
                      onClick={() => void onInstallLatest(row.id)}
                    >
                      安装最新版
                    </button>
                  ) : null}
                  <button type="button" className="ghost-button" onClick={onNavigateLogs}>
                    查看日志
                  </button>
                </div>
              ),
            }))}
          />
        ) : null}

        {detectFailedRows.length > 0 ? (
          <StatusPanel
            tone="danger"
            eyebrow="检测失败"
            title="检测命令超时或返回异常"
            description="保留真实错误消息，并提供重新检测和日志入口。"
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

        {pathRows.length > 0 ? (
          <StatusPanel
            tone="warning"
            eyebrow="PATH 缺失"
            title="可执行文件存在，但终端 PATH 还没刷新"
            description="本客户端不会自动修改 PATH。你可以先重启终端，或查看手动修复说明。"
            items={pathRows.map((row) => ({
              id: row.id,
              title: `${row.name} 需要确认 PATH`,
              description: row.executablePath ?? "未返回可执行文件路径",
              actions: (
                <div className="action-group">
                  <button
                    type="button"
                    className="ghost-button"
                    onClick={() => onOpenPathRepair(row)}
                  >
                    查看 PATH 修复说明
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
          title="安装说明"
          description="安装命令在内置终端中运行，输出会实时写入日志页。"
        >
          <div className="tool-info-grid">
            <div className="tool-info-item">
              <span>安装方式</span>
              <strong>npm install -g &lt;package&gt;</strong>
            </div>
            <div className="tool-info-item">
              <span>使用 npm 源</span>
              <strong>{npmRegistryLabel}</strong>
            </div>
            <div className="tool-info-item">
              <span>安装网络</span>
              <strong>{formatInstallNetwork(config)}</strong>
            </div>
          </div>
        </Card>

        <Card
          title="相关入口"
          description="ccSwitch 与订阅页继续复用现有打开逻辑，不改原有 invoke 链路。"
        >
          <div className="action-list">
            <div className="action-list-item">
              <div>
                <strong>ccSwitch</strong>
                <p>用于打开已有 GUI，并保留手动路径配置能力。</p>
              </div>
              <button
                type="button"
                className="ghost-button"
                onClick={() => void onOpenCcSwitch()}
                disabled={!canOpenCcSwitch}
              >
                打开
              </button>
            </div>
            <div className="action-list-item">
              <div>
                <strong>订阅页</strong>
                <p>通过当前配置中的 URL 在默认浏览器中打开。</p>
              </div>
              <button
                type="button"
                className="ghost-button"
                onClick={() => void onOpenSubscriptionPage()}
                disabled={!config.subscriptionPageUrl}
              >
                打开
              </button>
            </div>
          </div>
        </Card>
      </div>
    </div>
  );
}

function buildAiToolRowNote(row: ToolStatus): string | undefined {
  if (row.errorMessage) {
    return row.errorMessage;
  }

  if (row.id === "opencode" && row.status === "broken") {
    return "命令存在，但版本检测失败。建议重新安装。";
  }

  if (
    row.suggestion &&
    !SUPPRESSED_NOTE_PATTERNS.some((pattern) => row.suggestion?.includes(pattern))
  ) {
    return row.suggestion;
  }

  return undefined;
}

function formatInstallNetwork(config: AppConfig): string {
  if (config.installNetwork.mode === "none") {
    return "不使用代理 (none)";
  }

  if (config.installNetwork.mode === "system_proxy") {
    return "跟随系统代理 (system_proxy)";
  }

  return config.installNetwork.proxyUrl
    ? `手动代理 (${config.installNetwork.proxyUrl})`
    : "手动代理 (manual_proxy)";
}

export default AiToolsPage;
