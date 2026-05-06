import StatusPanel from "./StatusPanel";

type InstallActivityPanelProps = {
  toolName: string;
  latestLogLine?: string;
  onCancelInstall: () => Promise<void>;
  onNavigateLogs: () => void;
};

function InstallActivityPanel({
  toolName,
  latestLogLine,
  onCancelInstall,
  onNavigateLogs,
}: InstallActivityPanelProps) {
  return (
    <StatusPanel
      tone="info"
      eyebrow="安装中"
      title={`正在安装 ${toolName}`}
      description="正在安装单个工具，其他安装入口会保持禁用，避免并发安装。"
      actions={
        <div className="action-group">
          <button type="button" className="ghost-button" onClick={onNavigateLogs}>
            跳到日志
          </button>
          <button type="button" onClick={() => void onCancelInstall()}>
            取消安装
          </button>
        </div>
      }
      items={[
        {
          id: "phase",
          title: "运行中",
          description: latestLogLine ?? "安装任务已启动，实时输出会持续写入日志页。",
          meta: "当前没有百分比字段，因此使用真实阶段与日志片段展示。",
        },
      ]}
    />
  );
}

export default InstallActivityPanel;
