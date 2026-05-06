import { useEffect, useState } from "react";
import AdminStatePanel from "../components/status/AdminStatePanel";
import Card from "../components/ui/Card";
import type {
  AppConfig,
  InstallNetworkConfig,
  InstallNetworkMode,
  NpmRegistryOption,
} from "../types/config";

type SettingsPageProps = {
  config: AppConfig;
  adminChecked: boolean;
  isAdmin: boolean;
  onSaveInstallNetwork: (nextConfig: InstallNetworkConfig) => Promise<void>;
  onRestartAsAdmin: () => Promise<void>;
};

function SettingsPage({
  config,
  adminChecked,
  isAdmin,
  onSaveInstallNetwork,
  onRestartAsAdmin,
}: SettingsPageProps) {
  const [mode, setMode] = useState<InstallNetworkMode>(config.installNetwork.mode);
  const [npmRegistry, setNpmRegistry] = useState<NpmRegistryOption>(
    config.installNetwork.npmRegistry,
  );
  const [proxyUrl, setProxyUrl] = useState(config.installNetwork.proxyUrl ?? "");
  const [customRegistry, setCustomRegistry] = useState(
    config.installNetwork.customNpmRegistry ?? "",
  );
  const [saveMessage, setSaveMessage] = useState("");

  useEffect(() => {
    setMode(config.installNetwork.mode);
    setNpmRegistry(config.installNetwork.npmRegistry);
    setProxyUrl(config.installNetwork.proxyUrl ?? "");
    setCustomRegistry(config.installNetwork.customNpmRegistry ?? "");
  }, [config]);

  async function handleSave() {
    await onSaveInstallNetwork({
      mode,
      proxyUrl: mode === "manual_proxy" ? proxyUrl.trim() : undefined,
      npmRegistry,
      customNpmRegistry:
        npmRegistry === "custom" ? customRegistry.trim() || undefined : undefined,
    });
    setSaveMessage("安装网络设置已保存。");
  }

  return (
    <div className="tool-page-layout">
      <div className="page-stack">
        <AdminStatePanel
          adminChecked={adminChecked}
          isAdmin={isAdmin}
          onRestartAsAdmin={onRestartAsAdmin}
        />

        <section className="page-section panel" role="region" aria-label="设置">
          <div className="panel-header">
            <div>
              <h2>安装网络</h2>
              <p>
                只影响本客户端发起的安装命令。winget 是否读取代理，仍由 Windows
                系统环境决定。
              </p>
            </div>
            <div className="action-group">
              <button type="button" className="ghost-button" onClick={() => void handleSave()}>
                保存设置
              </button>
            </div>
          </div>

          <div className="settings-form-grid">
            <div className="form-field">
              <span>代理模式</span>
              <div className="choice-grid">
                <ChoiceCard
                  checked={mode === "none"}
                  title="不使用代理"
                  description="直连 npm registry / winget"
                  onSelect={() => setMode("none")}
                />
                <ChoiceCard
                  checked={mode === "system_proxy"}
                  title="使用系统代理"
                  description="读取 Windows 系统设置"
                  onSelect={() => setMode("system_proxy")}
                />
                <ChoiceCard
                  checked={mode === "manual_proxy"}
                  title="手动代理"
                  description="自定义 http(s):// 地址"
                  onSelect={() => setMode("manual_proxy")}
                />
              </div>
            </div>

            {mode === "manual_proxy" ? (
              <label className="form-field" htmlFor="proxy-url-input">
                <span>代理 URL</span>
                <input
                  id="proxy-url-input"
                  type="url"
                  value={proxyUrl}
                  onChange={(event) => setProxyUrl(event.target.value)}
                  placeholder="http://127.0.0.1:7890"
                />
              </label>
            ) : null}

            <label className="form-field" htmlFor="npm-registry-select">
              <span>npm Registry</span>
              <select
                id="npm-registry-select"
                value={npmRegistry}
                onChange={(event) => setNpmRegistry(event.target.value as NpmRegistryOption)}
              >
                <option value="default">默认</option>
                <option value="npmmirror">npmmirror</option>
                <option value="custom">自定义</option>
              </select>
            </label>

            {npmRegistry === "custom" ? (
              <label className="form-field" htmlFor="custom-registry-input">
                <span>自定义 registry URL</span>
                <input
                  id="custom-registry-input"
                  type="url"
                  value={customRegistry}
                  onChange={(event) => setCustomRegistry(event.target.value)}
                  placeholder="https://registry.example.com/"
                />
              </label>
            ) : null}

            <p className="network-warning">
              winget 的代理由 Windows 系统决定，本客户端不会接管系统代理。
            </p>
          </div>

          {saveMessage ? (
            <p className="inline-message" role="status">
              {saveMessage}
            </p>
          ) : null}
        </section>

        <Card
          title="管理员权限"
          description="当前会话的权限状态来自 is_admin / restart_as_admin 的真实调用链。"
        >
          <div className="ccswitch-status-grid">
            <div className="network-stat">
              <span>当前权限</span>
              <strong>{adminChecked ? (isAdmin ? "管理员" : "非管理员") : "检测中"}</strong>
            </div>
            <div className="network-stat">
              <span>提权方式</span>
              <strong>Windows UAC</strong>
            </div>
          </div>
        </Card>
      </div>

      <div className="tool-page-sidebar">
        <Card
          title="安全边界"
          description="这些限制是产品边界，当前不能在设置中关闭。"
        >
          <ul className="boundary-list">
            <li>不保存 API Key。</li>
            <li>不自动写入 Provider 配置。</li>
            <li>不接管系统代理。</li>
            <li>不自动修改 PATH。</li>
          </ul>
        </Card>

        <Card
          title="关于"
          description="设计稿中的版本与路径信息目前后端没有暴露为实时字段。"
        >
          {/* Runtime version/build/log-path metadata is not exposed by the current backend yet. */}
          <div className="tool-info-grid">
            <div className="tool-info-item">
              <span>配置来源</span>
              <strong>get_config / update_config</strong>
            </div>
            <div className="tool-info-item">
              <span>管理员检测</span>
              <strong>is_admin / restart_as_admin</strong>
            </div>
            <div className="tool-info-item">
              <span>运行时元数据</span>
              <strong>暂未由后端提供，当前仅保留说明卡片</strong>
            </div>
          </div>
        </Card>
      </div>
    </div>
  );
}

type ChoiceCardProps = {
  checked: boolean;
  title: string;
  description: string;
  onSelect: () => void;
};

function ChoiceCard({ checked, title, description, onSelect }: ChoiceCardProps) {
  return (
    <button
      type="button"
      className={`choice-card${checked ? " is-selected" : ""}`}
      onClick={onSelect}
    >
      <span className="choice-card-title">{title}</span>
      <span className="choice-card-description">{description}</span>
    </button>
  );
}

export default SettingsPage;
