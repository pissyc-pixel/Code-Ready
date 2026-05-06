import type { ReactNode } from "react";

type AlertBannerProps = {
  tone?: "info" | "warning";
  title: string;
  children: ReactNode;
  actions?: ReactNode;
};

function AlertBanner({
  tone = "info",
  title,
  children,
  actions,
}: AlertBannerProps) {
  return (
    <section className={`alert-banner ${tone}`} role="status">
      <div>
        <strong>{title}</strong>
        <p>{children}</p>
      </div>
      {actions ? <div className="alert-banner-actions">{actions}</div> : null}
    </section>
  );
}

export default AlertBanner;
