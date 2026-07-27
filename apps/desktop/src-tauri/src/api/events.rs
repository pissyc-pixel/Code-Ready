use crate::application::detection::{DetectionEventSink, EventPublishError};
use crate::domain::contracts::DetectionEventEnvelope;
use tauri::{AppHandle, Emitter, Runtime};

pub const DETECTION_CHANGED_EVENT: &str = "detection.changed";

pub struct TauriDetectionEventSink<R: Runtime> {
    app_handle: AppHandle<R>,
}

impl<R: Runtime> TauriDetectionEventSink<R> {
    pub fn new(app_handle: AppHandle<R>) -> Self {
        Self { app_handle }
    }
}

impl<R: Runtime> DetectionEventSink for TauriDetectionEventSink<R> {
    fn publish(&self, event: DetectionEventEnvelope) -> Result<(), EventPublishError> {
        self.app_handle
            .emit(DETECTION_CHANGED_EVENT, event)
            .map_err(|_| EventPublishError)
    }
}

#[cfg(test)]
mod tests {
    use super::DETECTION_CHANGED_EVENT;
    use crate::domain::contracts::{DetectionEventEnvelope, DetectionEventType};

    #[test]
    fn detection_changed_event_is_stable_and_metadata_only() {
        let event = DetectionEventEnvelope {
            schema_version: 1,
            sequence: 2,
            snapshot_version: 3,
            emitted_at_epoch_ms: 1_754_000_000_000,
            event_type: DetectionEventType::DetectionChanged,
            run_id: "run-1".to_owned(),
        };
        let payload = serde_json::to_value(event).expect("event serializes");

        assert_eq!(DETECTION_CHANGED_EVENT, "detection.changed");
        assert!(payload.get("observations").is_none());
        assert_eq!(payload.as_object().expect("event is an object").len(), 6);
    }
}
