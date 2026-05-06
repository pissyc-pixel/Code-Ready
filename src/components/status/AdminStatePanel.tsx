import StatusPanel from "./StatusPanel";

type AdminStatePanelProps = {
  adminChecked: boolean;
  isAdmin: boolean;
  onRestartAsAdmin: () => Promise<void>;
};

function AdminStatePanel({
  adminChecked,
  isAdmin,
  onRestartAsAdmin,
}: AdminStatePanelProps) {
  if (!adminChecked) {
    return (
      <StatusPanel
        tone="info"
        eyebrow="权限状态"
        title="正在确认当前权限"
        description="应用正在调用真实的 is_admin 检测当前会话权限。"
      />
    );
  }

  if (isAdmin) {
    return (
      <StatusPanel
        tone="success"
        eyebrow="权限状态"
        title="管理员权限已确认"
        description="系统范围安装仍走现有真实逻辑，这里只做状态确认，不改变后端行为。"
      />
    );
  }

  return (
    <StatusPanel
      tone="warning"
      eyebrow="权限提示"
      title="当前为标准用户运行"
      description="标准用户也能继续使用，但安装系统范围工具时可能触发 Windows UAC。"
      actions={
        <button type="button" onClick={() => void onRestartAsAdmin()}>
          以管理员身份重启
        </button>
      }
      items={[
        {
          id: "continue",
          title: "可继续的操作",
          description:
            "检测全部工具、查看日志、导出诊断 zip、保存 ccSwitch 路径与订阅页 URL、按用户范围安装 npm 全局包。",
        },
        {
          id: "needs-uac",
          title: "可能触发 UAC 的操作",
          description: "安装 Git / Node / Python（系统范围）、写入用户 PATH（需手动执行）。",
        },
      ]}
    />
  );
}

export default AdminStatePanel;
