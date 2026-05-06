import LogViewer from "../components/logs/LogViewer";
import AlertBanner from "../components/ui/AlertBanner";
import Card from "../components/ui/Card";

type LogsPageProps = {
  logLines: string[];
  logActionMessage: string;
  isExportingDiagnostics: boolean;
  logViewerLineLimit: number;
  onRefreshPreview: () => Promise<void>;
  onOpenFullLogFile: () => Promise<void>;
  onOpenLogDirectory: () => Promise<void>;
  onExportDiagnostics: () => Promise<void>;
};

function LogsPage({
  logLines,
  logActionMessage,
  isExportingDiagnostics,
  logViewerLineLimit,
  onRefreshPreview,
  onOpenFullLogFile,
  onOpenLogDirectory,
  onExportDiagnostics,
}: LogsPageProps) {
  const actionTone =
    logActionMessage.includes("失败") || logActionMessage.includes("不可用")
      ? "warning"
      : "info";

  return (
    <div className="tool-page-layout">
      <div className="page-stack">
        <section className="page-section panel" role="region" aria-label="日志">
          <div className="panel-header">
            <div>
              <h2>日志</h2>
              <p>
                存在但不喧宾夺主：检测和安装的实时输出、完整日志文件、诊断 zip 导出。
              </p>
            </div>
            <button
              type="button"
              onClick={() => void onExportDiagnostics()}
              disabled={isExportingDiagnostics}
            >
              {isExportingDiagnostics ? "导出中..." : "导出诊断 zip"}
            </button>
          </div>

          <div className="log-toolbar-card">
            <div className="log-toolbar-head">
              <strong>日志</strong>
              <span className="log-count">显示最近 {logLines.length} / {logViewerLineLimit} 行</span>
            </div>
            <div className="log-filter-hint">按级别与来源自动着色</div>
            <div className="action-group">
              <button type="button" className="ghost-button" onClick={() => void onRefreshPreview()}>
                刷新预览
              </button>
              <button type="button" className="ghost-button" onClick={() => void onOpenFullLogFile()}>
                打开完整日志
              </button>
              <button type="button" className="ghost-button" onClick={() => void onOpenLogDirectory()}>
                打开日志目录
              </button>
            </div>
          </div>

          {logActionMessage ? (
            <AlertBanner tone={actionTone} title="最近操作">
              {logActionMessage}
            </AlertBanner>
          ) : null}

          <LogViewer lines={logLines} />
        </section>
      </div>

      <div className="tool-page-sidebar">
        <Card
          title="诊断 zip 包含什么"
          description="导出前会展示文件清单。不会包含 API Key、token、subscriptionPageUrl 的查询参数。"
        >
          <div className="tool-info-grid">
            <div className="tool-info-item">
              <span>install.log</span>
              <strong>安装命令的实时输出 · 最近 5,000 行</strong>
            </div>
            <div className="tool-info-item">
              <span>detect.log</span>
              <strong>工具检测的结果与失败原因</strong>
            </div>
            <div className="tool-info-item">
              <span>config.redacted.json</span>
              <strong>脱敏后的本地配置（路径 / 网络模式）</strong>
            </div>
            <div className="tool-info-item">
              <span>env.txt</span>
              <strong>OS 版本、PATH 摘要、winget / npm 是否可用</strong>
            </div>
          </div>
        </Card>
      </div>
    </div>
  );
}

export default LogsPage;
