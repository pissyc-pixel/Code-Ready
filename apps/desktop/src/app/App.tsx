import { useEffect, useState } from "react";

import type { BootstrapState } from "../shared/api/generated";
import { getBootstrapState } from "../shared/api/client";
import { messages } from "../shared/i18n/zh-CN";

function platformLabel(platform: BootstrapState["platform"]): string {
  switch (platform) {
    case "windowsX64":
      return messages.platformWindowsX64;
    case "macosArm64":
      return messages.platformMacosArm64;
    default:
      return messages.platformUnknown;
  }
}

export default function App() {
  const [bootstrap, setBootstrap] = useState<BootstrapState | null>(null);
  const [hasError, setHasError] = useState(false);

  useEffect(() => {
    let active = true;

    void getBootstrapState()
      .then((state) => {
        if (active) {
          setBootstrap(state);
        }
      })
      .catch(() => {
        if (active) {
          setHasError(true);
        }
      });

    return () => {
      active = false;
    };
  }, []);

  if (hasError) {
    return (
      <main className="app-shell">
        <p className="status-message">{messages.bootstrapError}</p>
      </main>
    );
  }

  if (!bootstrap) {
    return (
      <main className="app-shell">
        <p className="status-message">{messages.loadingBootstrap}</p>
      </main>
    );
  }

  return (
    <main className="app-shell">
      <header className="hero">
        <p className="eyebrow">{messages.baseline}</p>
        <h1>{messages.appTitle}</h1>
      </header>

      <section className="summary-grid" aria-label={messages.platformHeading}>
        <article className="summary-card">
          <p className="card-label">{messages.platformHeading}</p>
          <p className="card-value">{platformLabel(bootstrap.platform)}</p>
        </article>
        <article className="summary-card">
          <p className="card-label">{messages.registryHeading}</p>
          <p className="card-value">{messages.toolCount(bootstrap.tools.length)}</p>
        </article>
      </section>

      <p className="next-slice">{messages.nextSlicePlaceholder}</p>
    </main>
  );
}
