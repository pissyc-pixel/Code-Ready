import { useDetectionSnapshot } from "../features/detection/useDetectionSnapshot";
import type { AppSnapshot } from "../shared/api/generated";
import { messages } from "../shared/i18n/zh-CN";

function platformLabel(platform: AppSnapshot["platform"]): string {
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
  const { phase, snapshot } = useDetectionSnapshot();

  if (phase === "bootstrapError") {
    return (
      <main className="app-shell">
        <p className="status-message">{messages.bootstrapError}</p>
      </main>
    );
  }

  if (!snapshot) {
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
          <p className="card-value">{platformLabel(snapshot.platform)}</p>
        </article>
        <article className="summary-card">
          <p className="card-label">{messages.registryHeading}</p>
          <p className="card-value">{messages.toolCount(snapshot.tools.length)}</p>
        </article>
      </section>

      <p className="next-slice">{messages.nextSlicePlaceholder}</p>
    </main>
  );
}
