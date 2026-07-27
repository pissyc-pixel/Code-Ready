import { useState } from "react";

import Onboarding from "../features/onboarding/Onboarding";
import StatusCenter from "../features/dashboard/StatusCenter";
import {
  type DetectionSnapshotState,
  useDetectionSnapshot,
} from "../features/detection/useDetectionSnapshot";
import type { AppSnapshot } from "../shared/api/generated";
import { message } from "../shared/i18n/zh-CN";

function LoadingScreen() {
  return (
    <main className="app-shell">
      <p className="status-message">{message("loadingBootstrap")}</p>
    </main>
  );
}

function BootstrapErrorScreen({ onRetry }: { onRetry: () => Promise<void> }) {
  return (
    <main className="app-shell">
      <p className="status-message" role="alert">{message("bootstrapError")}</p>
      <button type="button" onClick={() => void onRetry()}>
        {message("actions.retry")}
      </button>
    </main>
  );
}

type ReadyDetectionSnapshotState = Omit<DetectionSnapshotState, "snapshot"> & {
  snapshot: AppSnapshot;
};

function ReadyApp({ detection }: { detection: ReadyDetectionSnapshotState }) {
  const [mode, setMode] = useState<"onboarding" | "dashboard">(
    () => (detection.snapshot.detectionRun?.status === "completed" ? "dashboard" : "onboarding"),
  );

  if (mode === "onboarding") {
    return (
      <Onboarding
        snapshot={detection.snapshot}
        startDetection={detection.startDetection}
        refresh={detection.refresh}
        syncWarning={detection.syncWarning}
        detectionStartError={detection.detectionStartError}
        onComplete={() => setMode("dashboard")}
      />
    );
  }

  return (
    <StatusCenter
      snapshot={detection.snapshot}
      startDetection={detection.startDetection}
      refresh={detection.refresh}
      syncWarning={detection.syncWarning}
      detectionStartError={detection.detectionStartError}
    />
  );
}

export default function App() {
  const detection = useDetectionSnapshot();

  if (detection.phase === "bootstrapError") {
    return <BootstrapErrorScreen onRetry={detection.refresh} />;
  }
  if (detection.phase === "loading" || detection.snapshot === null) {
    return <LoadingScreen />;
  }

  return <ReadyApp detection={detection as ReadyDetectionSnapshotState} />;
}
