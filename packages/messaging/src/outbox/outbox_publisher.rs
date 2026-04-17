//! OutboxPublisher — background poller that processes pending outbox entries.
//!
//! This runs in a background task, periodically querying for unprocessed
//! outbox entries and publishing them to the EventBus.
//!
//! This is the packages/messaging-owned publisher. The outbox-relay worker
//! has its own relay-specific publisher at `workers/outbox-relay/src/publish/`
//! that also dispatches to PubSub — both read from the same unified
//! `event_outbox` table.

use std::time::Duration;

use chrono::{DateTime, Utc};
use contracts_events::AppEvent;
use serde::Deserialize;
use tokio::time;
use tracing::{debug, error, info, warn};

use crate::outbox::outbox_entry::{
    MARK_FAILED_SQL, MARK_PUBLISHED_SQL, OutboxEntry, OutboxStatus, SELECT_PENDING_SQL,
};
use crate::ports::{EventBus, EventBusError, EventEnvelope};

/// Abstract database port for the outbox publisher.
/// This avoids depending on a specific DB implementation.
#[async_trait::async_trait]
pub trait OutboxStore: Send + Sync {
    /// Execute a SQL statement with parameters.
    async fn execute(
        &self,
        sql: &str,
        params: Vec<String>,
    ) -> Result<u64, Box<dyn std::error::Error + Send + Sync>>;

    /// Execute a query returning deserialized rows.
    async fn query<T: serde::de::DeserializeOwned + Send + Sync>(
        &self,
        sql: &str,
        params: Vec<String>,
    ) -> Result<Vec<T>, Box<dyn std::error::Error + Send + Sync>>;
}

/// Configuration for the outbox publisher.
#[derive(Debug, Clone)]
pub struct OutboxPublisherConfig {
    /// How often to poll for pending entries.
    pub poll_interval: Duration,
    /// Maximum retries per entry before giving up.
    pub max_retries: u32,
}

impl Default for OutboxPublisherConfig {
    fn default() -> Self {
        Self {
            poll_interval: Duration::from_secs(5),
            max_retries: 5,
        }
    }
}

/// Raw row from the event_outbox table.
#[derive(Debug, Deserialize)]
struct OutboxRow {
    sequence: i64,
    event_id: String,
    event_type: String,
    event_payload: String,
    source_service: String,
    correlation_id: Option<String>,
    status: String,
    created_at: String,
    published_at: Option<String>,
    retry_count: i64,
}

/// Outbox publisher — processes pending events in the background.
pub struct OutboxPublisher<S: OutboxStore, E: EventBus> {
    store: S,
    event_bus: E,
    config: OutboxPublisherConfig,
}

impl<S: OutboxStore, E: EventBus> OutboxPublisher<S, E> {
    pub fn new(store: S, event_bus: E, config: OutboxPublisherConfig) -> Self {
        Self {
            store,
            event_bus,
            config,
        }
    }

    /// Run the publisher loop. This never returns unless the task is cancelled.
    pub async fn run(self) {
        info!(
            "Outbox publisher starting (poll_interval: {:?})",
            self.config.poll_interval
        );

        let mut interval = time::interval(self.config.poll_interval);
        loop {
            interval.tick().await;
            if let Err(e) = self.process_pending().await {
                error!(error = %e, "outbox publisher error");
            }
        }
    }

    /// Process one batch of pending entries.
    async fn process_pending(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let rows: Vec<OutboxRow> = self.store.query(SELECT_PENDING_SQL, vec![]).await?;

        if rows.is_empty() {
            return Ok(());
        }

        debug!(count = rows.len(), "processing pending outbox entries");

        for row in rows {
            match self.process_entry(&row).await {
                Ok(()) => {
                    self.store
                        .execute(MARK_PUBLISHED_SQL, vec![row.event_id.clone()])
                        .await?;
                    debug!(sequence = row.sequence, event_id = %row.event_id, "outbox entry published");
                }
                Err(e) => {
                    warn!(event_id = %row.event_id, error = %e, "failed to publish outbox entry");
                    if row.retry_count as u32 >= self.config.max_retries {
                        error!(
                            event_id = %row.event_id,
                            retries = row.retry_count,
                            "outbox entry exceeded max retries, dropping"
                        );
                    } else {
                        self.store
                            .execute(MARK_FAILED_SQL, vec![row.event_id.clone()])
                            .await?;
                    }
                }
            }
        }

        Ok(())
    }

    async fn process_entry(&self, row: &OutboxRow) -> Result<(), EventBusError> {
        let event: AppEvent = serde_json::from_str(&row.event_payload)
            .map_err(|e| EventBusError::HandlerError(format!("deserialize: {e}")))?;

        let envelope = EventEnvelope::new(event, &row.source_service);
        self.event_bus.publish(envelope).await
    }
}
