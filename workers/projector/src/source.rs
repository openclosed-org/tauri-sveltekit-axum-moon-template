//! Projection source — reads replayable events from the unified event_outbox.

use contracts_events::{AppEvent, CounterChanged, EventEnvelope};
use data::ports::lib_sql::LibSqlPort;
use serde::Deserialize;

use crate::ProjectorError;

#[derive(Debug, Clone)]
pub struct ProjectionEvent {
    pub sequence: u64,
    pub envelope: event_bus::ports::EventEnvelope,
}

#[derive(Debug, Deserialize)]
struct OutboxRow {
    sequence: i64,
    event_payload: String,
    source_service: String,
}

/// Reads from the unified event_outbox table for projection replay.
///
/// All services write to this table, so a single source can replay
/// events from any service — no per-service outbox source needed.
pub struct CounterOutboxSource<P: LibSqlPort> {
    port: P,
}

impl<P: LibSqlPort> CounterOutboxSource<P> {
    pub fn new(port: P) -> Self {
        Self { port }
    }

    pub async fn fetch_since(
        &self,
        since_sequence: u64,
        limit: usize,
    ) -> Result<Vec<ProjectionEvent>, ProjectorError> {
        let rows: Vec<OutboxRow> = self
            .port
            .query(
                "SELECT sequence, event_payload, source_service \
                 FROM event_outbox \
                 WHERE sequence > ? \
                 ORDER BY sequence ASC \
                 LIMIT ?",
                vec![since_sequence.to_string(), limit.to_string()],
            )
            .await
            .map_err(|e| ProjectorError::Source(format!("query event_outbox: {e}")))?;

        rows.into_iter()
            .map(|row| {
                let envelope = deserialize_envelope(&row.event_payload)?;
                Ok(ProjectionEvent {
                    sequence: row.sequence as u64,
                    envelope: event_bus::ports::EventEnvelope {
                        source_service: row.source_service,
                        ..envelope
                    },
                })
            })
            .collect()
    }
}

fn deserialize_envelope(payload: &str) -> Result<EventEnvelope, ProjectorError> {
    match serde_json::from_str::<EventEnvelope>(payload) {
        Ok(envelope) => Ok(envelope),
        Err(envelope_error) => match serde_json::from_str::<AppEvent>(payload) {
            Ok(event) => Ok(EventEnvelope::new(event, "counter-service")),
            Err(app_event_error) => serde_json::from_str::<CounterChanged>(payload)
                .map(AppEvent::CounterChanged)
                .map(|event| EventEnvelope::new(event, "counter-service"))
                .map_err(|counter_changed_error| {
                    ProjectorError::Source(format!(
                        "deserialize event envelope: {envelope_error}; app event: {app_event_error}; counter-changed fallback: {counter_changed_error}"
                    ))
                }),
        },
    }
}
