import type { ReactNode } from "react";

type StatusPanelTone = "info" | "warning" | "danger" | "success";

type StatusPanelItem = {
  id: string;
  title: string;
  description: string;
  meta?: string;
  actions?: ReactNode;
};

type StatusPanelProps = {
  tone?: StatusPanelTone;
  eyebrow?: string;
  title: string;
  description: string;
  actions?: ReactNode;
  items?: StatusPanelItem[];
  children?: ReactNode;
};

function StatusPanel({
  tone = "info",
  eyebrow,
  title,
  description,
  actions,
  items,
  children,
}: StatusPanelProps) {
  return (
    <section className={`status-panel is-${tone}`}>
      <header className="status-panel-header">
        <div>
          {eyebrow ? <p className="card-eyebrow">{eyebrow}</p> : null}
          <h3 className="status-panel-title">{title}</h3>
          <p className="status-panel-description">{description}</p>
        </div>
        {actions ? <div className="status-panel-actions">{actions}</div> : null}
      </header>

      {items?.length ? (
        <div className="status-panel-list">
          {items.map((item) => (
            <div className="status-panel-item" key={item.id}>
              <div>
                <strong>{item.title}</strong>
                <p>{item.description}</p>
                {item.meta ? <span className="status-panel-meta">{item.meta}</span> : null}
              </div>
              {item.actions ? <div className="status-panel-item-actions">{item.actions}</div> : null}
            </div>
          ))}
        </div>
      ) : null}

      {children}
    </section>
  );
}

export type { StatusPanelItem, StatusPanelTone };
export default StatusPanel;
