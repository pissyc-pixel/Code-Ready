import type {
  AppSnapshot,
  CommandErrorCode,
  ObservedToolState,
  ToolDefinition,
  ToolId,
  ToolObservation,
} from "../../shared/api/generated";
import {
  formatObservedVersion,
  formatRedetectOne,
  message,
  toolLabel,
} from "../../shared/i18n/zh-CN";
import {
  factMessageKey,
  versionMessageKey,
} from "../detection/presentation";
import type { SyncWarning } from "../detection/useDetectionSnapshot";

export interface StatusCenterProps {
  snapshot: AppSnapshot;
  startDetection: (toolIds?: ToolId[]) => Promise<void>;
  refresh: () => Promise<void>;
  syncWarning: SyncWarning;
  detectionStartError: CommandErrorCode | null;
}

const SLICE_ONE_TOOL_IDS: ToolId[] = ["git", "claudeCode", "codexCli"];
const NEEDS_ATTENTION: ObservedToolState[] = [
  "absent",
  "presentPathIssue",
  "presentBroken",
  "unknown",
];

function platformLabel(platform: AppSnapshot["platform"]): string {
  return platform === "windowsX64"
    ? message("platformWindowsX64")
    : message("platformMacosArm64");
}

function labelFor(tool: ToolDefinition): string {
  return toolLabel(tool.labelKey);
}

function toolEntries(snapshot: AppSnapshot): Array<{
  tool: ToolDefinition;
  observation: ToolObservation | undefined;
}> {
  return SLICE_ONE_TOOL_IDS.flatMap((toolId) => {
    const tool = snapshot.tools.find((definition) => definition.id === toolId);
    return tool === undefined
      ? []
      : [{
          tool,
          observation: snapshot.observations.find((item) => item.toolId === toolId),
        }];
  });
}

function ObservationCard({
  tool,
  observation,
  activeRun,
  startDetection,
}: {
  tool: ToolDefinition;
  observation: ToolObservation | undefined;
  activeRun: boolean;
  startDetection: (toolIds?: ToolId[]) => Promise<void>;
}) {
  const label = labelFor(tool);
  return (
    <article className="status-card">
      <header className="status-card-header">
        <h3>{label}</h3>
        <button
          type="button"
          disabled={activeRun}
          onClick={() => void startDetection([tool.id])}
        >
          {formatRedetectOne(label)}
        </button>
      </header>
      {observation === undefined ? (
        <p className="fact-badge">{message("dashboard.notYetDetected")}</p>
      ) : (
        <>
          <p className="fact-badge">{message(factMessageKey(observation.state))}</p>
          {observation.version !== null && (
            <p className="version-value">{formatObservedVersion(observation.version)}</p>
          )}
          <p className="version-status">
            {message(versionMessageKey(observation.versionStatus))}
          </p>
          {observation.evidence.displayPath !== null && (
            <p className="path-detail">
              <span>{message("dashboard.path")}</span>{" "}
              <code>{observation.evidence.displayPath}</code>
            </p>
          )}
        </>
      )}
    </article>
  );
}

export default function StatusCenter({
  snapshot,
  startDetection,
  refresh,
  syncWarning,
  detectionStartError,
}: StatusCenterProps) {
  const entries = toolEntries(snapshot);
  const attention = entries.filter(
    ({ observation }) => observation === undefined || NEEDS_ATTENTION.includes(observation.state),
  );
  const healthy = entries.filter(
    ({ observation }) => observation?.state === "presentHealthy",
  );
  const activeRun = snapshot.detectionRun?.status === "running";

  return (
    <main className="app-shell dashboard-shell">
      <header className="dashboard-header">
        <div>
          <p className="eyebrow">{message("dashboard.readOnly")}</p>
          <h1>{message("dashboard.title")}</h1>
          <p className="platform-label">{platformLabel(snapshot.platform)}</p>
        </div>
        <div className="dashboard-actions">
          <button type="button" onClick={() => void refresh()}>
            {message("actions.refresh")}
          </button>
          <button
            type="button"
            disabled={activeRun}
            onClick={() => void startDetection(undefined)}
          >
            {message("actions.redetectAll")}
          </button>
        </div>
      </header>

      {activeRun && (
        <p className="run-status" role="status" aria-live="polite">
          {message("detection.running")}
        </p>
      )}
      {syncWarning === "eventUnavailable" && (
        <p className="sync-warning" role="status">
          {message("sync.eventUnavailable")}
        </p>
      )}
      {syncWarning === "refreshFailed" && (
        <p className="sync-warning" role="status">
          {message("sync.refreshFailed")}
        </p>
      )}
      {detectionStartError !== null && (
        <p className="sync-warning" role="alert">
          {message("detection.startError")}
        </p>
      )}

      <section aria-labelledby="dashboard-attention-title">
        <h2 id="dashboard-attention-title">{message("dashboard.needsAttention")}</h2>
        <div className="status-grid">
          {attention.map(({ tool, observation }) => (
            <ObservationCard
              key={tool.id}
              tool={tool}
              observation={observation}
              activeRun={activeRun}
              startDetection={startDetection}
            />
          ))}
        </div>
      </section>

      <section aria-labelledby="dashboard-healthy-title">
        <h2 id="dashboard-healthy-title">{message("dashboard.healthy")}</h2>
        <div className="status-grid">
          {healthy.map(({ tool, observation }) => (
            <ObservationCard
              key={tool.id}
              tool={tool}
              observation={observation}
              activeRun={activeRun}
              startDetection={startDetection}
            />
          ))}
        </div>
      </section>
    </main>
  );
}
