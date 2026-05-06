import Card from "../components/ui/Card";
import StatusBadge from "../components/ui/StatusBadge";
import AlertBanner from "../components/ui/AlertBanner";
import type { ToolStatus } from "../types/tool";

type CCSwitchPageProps = {
  row?: ToolStatus;
  ccswitchPathInput: string;
  subscriptionPageUrlInput: string;
  downloadSourceCount: number;
  quickActionMessage: string;
  onPathInputChange: (value: string) => void;
  onSubscriptionUrlInputChange: (value: string) => void;
  onSavePath: () => Promise<void>;
  onSaveSubscriptionPageUrl: () => Promise<void>;
  onOpenCcSwitch: () => Promise<void>;
  onOpenSubscriptionPage: () => Promise<void>;
  onDetectCcSwitch: () => Promise<void>;
};

function CCSwitchPage({
  row,
  ccswitchPathInput,
  subscriptionPageUrlInput,
  downloadSourceCount,
  quickActionMessage,
  onPathInputChange,
  onSubscriptionUrlInputChange,
  onSavePath,
  onSaveSubscriptionPageUrl,
  onOpenCcSwitch,
  onOpenSubscriptionPage,
  onDetectCcSwitch,
}: CCSwitchPageProps) {
  const canOpenCcSwitch = Boolean(row?.executablePath);
  const canOpenSubscriptionPage = subscriptionPageUrlInput.trim().startsWith("http");

  return (
    <div className="tool-page-layout">
      <div className="page-stack">
        <section className="page-section panel" role="region" aria-label="ccSwitch">
          <div className="panel-header">
            <div>
              <h2>ccSwitch</h2>
              <p>
                启动入口、手动路径、节点订阅网页 URL 都只负责保存和打开，不解析订阅，
                也不接管代理。
              </p>
            </div>
            <div className="action-group">
              <button
                type="button"
                className="ghost-button"
                onClick={() => void onDetectCcSwitch()}
              >
                重新检测
              </button>
              <button
                type="button"
                onClick={() => void onOpenCcSwitch()}
                disabled={!canOpenCcSwitch}
              >
                打开 ccSwitch
              </button>
            </div>
          </div>

          <div className="ccswitch-status-grid">
            <div className="network-stat">
              <span>当前状态</span>
              <div className="status-inline">
                <StatusBadge status={row?.status ?? "missing"} />
                <strong>{row?.version ?? "未检测到版本"}</strong>
              </div>
            </div>
            <div className="network-stat">
              <span>检测结果</span>
              <strong className="mono breakable">
                {row?.executablePath ?? "尚未检测到可执行文件"}
              </strong>
            </div>
          </div>

          {row?.errorMessage ? (
            <AlertBanner tone="warning" title="检测提示">
              {row.errorMessage}
            </AlertBanner>
          ) : null}

          {quickActionMessage ? (
            <AlertBanner tone="info" title="最近操作">
              {quickActionMessage}
            </AlertBanner>
          ) : null}
        </section>

        <Card
          title="ccSwitch 路径"
          description="手动指定可执行文件位置。检测只做路径探测，不会自动启动 GUI。"
        >
          <div className="settings-form-grid">
            <label className="form-field" htmlFor="ccswitch-path-input">
              <span>可执行文件路径</span>
              <input
                id="ccswitch-path-input"
                type="text"
                value={ccswitchPathInput}
                onChange={(event) => onPathInputChange(event.target.value)}
                placeholder="C:\\Program Files\\ccswitch\\ccswitch.exe"
              />
            </label>

            <div className="action-group">
              <button type="button" onClick={() => void onSavePath()}>
                保存
              </button>
              <button
                type="button"
                className="ghost-button"
                onClick={() => void onDetectCcSwitch()}
              >
                保存后重新检测
              </button>
              <button type="button" className="ghost-button" disabled>
                浏览…
              </button>
            </div>
            {/* Backend does not currently expose a native file picker for ccSwitch path selection. */}
            <p className="tool-detail">
              已配置下载源 {downloadSourceCount} 个。当所有源失败时，建议手动指定路径。
            </p>
          </div>
        </Card>

        <Card
          title="节点订阅网页"
          description="这里只保存和打开 URL，不解析订阅，不下载节点，不设置代理。"
        >
          <div className="settings-form-grid">
            <label className="form-field" htmlFor="subscription-page-url-input">
              <span>订阅页 URL</span>
              <input
                id="subscription-page-url-input"
                type="url"
                value={subscriptionPageUrlInput}
                onChange={(event) => onSubscriptionUrlInputChange(event.target.value)}
                placeholder="https://example.com/dashboard"
              />
            </label>
            <div className="action-group">
              <button type="button" onClick={() => void onSaveSubscriptionPageUrl()}>
                保存
              </button>
              <button
                type="button"
                className="ghost-button"
                onClick={() => void onOpenSubscriptionPage()}
                disabled={!canOpenSubscriptionPage}
              >
                打开
              </button>
            </div>
            <p className="tool-detail">
              只接受 `http://` 或 `https://` 开头。打开会调用系统默认浏览器。
            </p>
          </div>
        </Card>
      </div>

      <div className="tool-page-sidebar">
        <Card
          title="路径与启动"
          description="设计稿里的说明被保留为真实操作面板，按钮继续复用现有 invoke。"
        >
          <div className="tool-info-grid">
            <div className="tool-info-item">
              <span>打开入口</span>
              <strong>open_ccswitch</strong>
            </div>
            <div className="tool-info-item">
              <span>订阅页入口</span>
              <strong>open_subscription_page</strong>
            </div>
            <div className="tool-info-item">
              <span>保存逻辑</span>
              <strong>update_config → detect_tool(ccswitch)</strong>
            </div>
          </div>
        </Card>
      </div>
    </div>
  );
}

export default CCSwitchPage;
