import type { ReactNode } from "react";

type AppShellProps = {
  sidebar: ReactNode;
  topbar: ReactNode;
  children: ReactNode;
};

function AppShell({ sidebar, topbar, children }: AppShellProps) {
  return (
    <div className="app-shell">
      {sidebar}
      <div className="app-main">
        {topbar}
        <main className="app-content">{children}</main>
      </div>
    </div>
  );
}

export default AppShell;
