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

type AiToolsPageProps = {
  rows: ToolStatus[];
  config: AppConfig;
  isDetectingAll: boolean;
  installState: InstallTaskViewState;
  canOpenCcSwitch: boolean;
  onDetectAll: () => Promise<void>;
  onDetect: (toolId: ToolId) => Promise<void>;
  onInstall: (toolId: ToolId) => Promise<void>;
  onReinstall: (toolId: ToolId) => Promise<void>;
  onInstallLatest: (toolId: ToolId) => Promise<void>;
  onCancelInstall: () => Promise<void>;
  onOpenPathRepair: (tool: ToolStatus) => void;
  onOpenCcSwitch: () => Promise<void>;
  onOpenSubscriptionPage: () => Promise<void>;
};

function AiToolsPage({
  rows,
  config,
  isDetectingAll,
  installState,
  canOpenCcSwitch,
  onDetectAll,
  onDetect,
  onInstall,
  onReinstall,
  onInstallLatest,
  onCancelInstall,
  onOpenPathRepair,
  onOpenCcSwitch,
  onOpenSubscriptionPage,
}: AiToolsPageProps) {
  const npmRegistryLabel =
    config.installNetwork.npmRegistry === "custom"
      ? config.installNetwork.customNpmRegistry ?? "custom"
      : config.installNetwork.npmRegistry;

  return (
    <div className="tool-page-layout">
      <section className="page-section panel" role="region" aria-label="AI 工具">
        <div className="panel-header">
          <div>
            <h2>AI 工具</h2>
            <p>
              Claude Code · Codex CLI · OpenCode · ccSwitch 的检测、安装、更新与取消逻辑全部保留。
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
          每个工具单独安装，单独取消。安装最新版只对支持 npm -g 的工具开放。
        </p>

        <ToolTable
          rows={rows}
          installState={installState}
          subtitles={AI_TOOL_SUBTITLES}
          getRowNote={buildAiToolRowNote}
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
          title="安装说明"
          description="安装命令在内置终端中运行，输出实时写入日志页。"
        >
          <div className="tool-info-grid">
            <div className="tool-info-item">
              <span>安装方式</span>
              <strong>npm install -g &lt;package&gt;</strong>
            </div>
            <div className="tool-info-item">
              <span>使用 registry</span>
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
          description="设计稿里的 ccSwitch 与订阅入口，继续复用现有 invoke，不改后端。"
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
                <p>仍然通过当前配置中的 URL 在默认浏览器中打开。</p>
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
    return "命令存在，但版本探测失败。建议重新安装。";
  }

  return row.suggestion;
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
