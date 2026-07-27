import { useState } from "react";

import type {
  AppSnapshot,
  CommandErrorCode,
  ToolId,
} from "../../shared/api/generated";
import {
  formatObservedVersion,
  message,
  toolLabel,
} from "../../shared/i18n/zh-CN";
import {
  factMessageKey,
  versionMessageKey,
} from "../detection/presentation";

export interface OnboardingProps {
  snapshot: AppSnapshot;
  startDetection: (toolIds?: ToolId[]) => Promise<void>;
  onComplete: () => void;
  detectionStartError?: CommandErrorCode | null;
}

type OnboardingStep = "welcome" | "detection" | "explanation" | "failed";

function stepFor(snapshot: AppSnapshot): OnboardingStep {
  const run = snapshot.detectionRun;
  if (run === null) {
    return "welcome";
  }
  if (run.status === "running") {
    return "detection";
  }
  if (run.status === "failed") {
    return "failed";
  }
  const hasRunObservation = snapshot.observations.some((observation) =>
    run.requestedToolIds.includes(observation.toolId),
  );
  return hasRunObservation ? "explanation" : "detection";
}

function ObservationSummary({ snapshot }: { snapshot: AppSnapshot }) {
  return (
    <ul aria-label={message("onboarding.explain.title")}>
      {snapshot.observations.map((observation) => {
        const tool = snapshot.tools.find((definition) => definition.id === observation.toolId);
        const label = toolLabel(tool?.labelKey ?? "");
        return (
          <li key={observation.toolId}>
            <span>{label}</span>
            <span>{message(factMessageKey(observation.state))}</span>
            {observation.version !== null && (
              <span>{formatObservedVersion(observation.version)}</span>
            )}
            <span>{message(versionMessageKey(observation.versionStatus))}</span>
          </li>
        );
      })}
    </ul>
  );
}

export default function Onboarding({
  snapshot,
  startDetection,
  onComplete,
  detectionStartError = null,
}: OnboardingProps) {
  const [startPending, setStartPending] = useState(false);
  const step = stepFor(snapshot);

  const beginDetection = async () => {
    setStartPending(true);
    try {
      await startDetection(undefined);
    } finally {
      setStartPending(false);
    }
  };

  return (
    <main className="app-shell">
      <header className="hero">
        <p className="eyebrow">{message("app.title")}</p>
        <h1>{message("app.title")}</h1>
      </header>

      {step === "welcome" && (
        <section aria-labelledby="onboarding-welcome-title">
          <h2 id="onboarding-welcome-title">{message("onboarding.welcome.title")}</h2>
          <p>{message("onboarding.welcome.body")}</p>
          <p>{message("privacy.zeroTelemetry")}</p>
          <button type="button" disabled={startPending} onClick={() => void beginDetection()}>
            {message("onboarding.start")}
          </button>
        </section>
      )}

      {step === "detection" && (
        <section aria-labelledby="onboarding-detection-title">
          <h2 id="onboarding-detection-title">{message("onboarding.detecting.title")}</h2>
          <p role="status" aria-live="polite">{message("detection.running")}</p>
        </section>
      )}

      {step === "failed" && (
        <section aria-labelledby="onboarding-failed-title">
          <h2 id="onboarding-failed-title">{message("onboarding.detecting.title")}</h2>
          <p role="alert">{message("detection.runFailed")}</p>
          <ObservationSummary snapshot={snapshot} />
          <button type="button" disabled={startPending} onClick={() => void beginDetection()}>
            {message("actions.retry")}
          </button>
        </section>
      )}

      {step === "explanation" && (
        <section aria-labelledby="onboarding-explanation-title">
          <h2 id="onboarding-explanation-title">{message("onboarding.explain.title")}</h2>
          <ObservationSummary snapshot={snapshot} />
          <button type="button" onClick={onComplete}>
            {message("onboarding.enterStatusCenter")}
          </button>
        </section>
      )}

      {detectionStartError !== null && (
        <p role="alert">
          {message("detection.startError")}
          <button type="button" disabled={startPending} onClick={() => void beginDetection()}>
            {message("actions.retry")}
          </button>
        </p>
      )}
    </main>
  );
}
