type SideNavItem = {
  id: string;
  label: string;
  caption?: string;
};

type SideNavProps = {
  items: SideNavItem[];
  activeItemId: string;
  onSelect: (id: string) => void;
};

function SideNav({ items, activeItemId, onSelect }: SideNavProps) {
  return (
    <aside className="side-nav">
      <div className="side-nav-brand">
        <p className="side-nav-kicker">v0.5 · Windows</p>
        <h1>AI Coding 环境助手</h1>
        <p className="side-nav-copy">Workspace</p>
      </div>

      <nav className="side-nav-menu" aria-label="应用导航">
        {items.map((item) => {
          const isActive = item.id === activeItemId;
          return (
            <button
              key={item.id}
              type="button"
              className={`nav-button${isActive ? " is-active" : ""}`}
              aria-current={isActive ? "page" : undefined}
              onClick={() => onSelect(item.id)}
            >
              <span className="nav-button-label">{item.label}</span>
              {item.caption ? <span className="nav-button-caption">{item.caption}</span> : null}
            </button>
          );
        })}
      </nav>
    </aside>
  );
}

export type { SideNavItem };
export default SideNav;
