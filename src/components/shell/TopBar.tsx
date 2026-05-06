type TopBarProps = {
  title: string;
  description: string;
  npmRegistryLabel: string;
  isAdmin: boolean;
  activeInstallLabel?: string;
};

function TopBar({
  title,
  description,
  npmRegistryLabel,
  isAdmin,
  activeInstallLabel,
}: TopBarProps) {
  return (
    <header className="top-bar">
      <div>
        <p className="top-bar-eyebrow">本机环境</p>
        <h2 className="top-bar-title">{title}</h2>
        <p className="top-bar-description">{description}</p>
      </div>

      <div className="top-bar-meta">
        <span className="top-bar-pill">npm 源 · {npmRegistryLabel}</span>
        <span className={`top-bar-pill ${isAdmin ? "is-success" : "is-warning"}`}>
          {isAdmin ? "管理员" : "标准用户"}
        </span>
        {activeInstallLabel ? (
          <span className="top-bar-pill is-info">安装中 · {activeInstallLabel}</span>
        ) : null}
      </div>
    </header>
  );
}

export default TopBar;
