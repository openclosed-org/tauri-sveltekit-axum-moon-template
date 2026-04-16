//! Live tail support for projector via NATS subject subscription.

use std::time::Duration;

use async_nats::Subscriber;
use contracts_events::{AppEvent, EventEnvelope};
use futures_util::StreamExt;
use tokio::time::timeout;

use crate::ProjectorError;

pub struct LiveEventSubscriber {
    subscriber: Subscriber,
}

impl LiveEventSubscriber {
    pub async fn connect(
        nats_url: &str,
        subject: &str,
        queue_group: Option<&str>,
    ) -> Result<Self, ProjectorError> {
        let client = async_nats::connect(nats_url)
            .await
            .map_err(|e| ProjectorError::Source(format!("connect nats: {e}")))?;
        let subscriber = if let Some(queue_group) = queue_group.filter(|value| !value.is_empty()) {
            client
                .queue_subscribe(subject.to_string(), queue_group.to_string())
                .await
                .map_err(|e| {
                    ProjectorError::Source(format!(
                        "queue subscribe '{subject}' group '{queue_group}': {e}"
                    ))
                })?
        } else {
            client
                .subscribe(subject.to_string())
                .await
                .map_err(|e| ProjectorError::Source(format!("subscribe '{subject}': {e}")))?
        };

        Ok(Self { subscriber })
    }

    pub async fn try_next(
        &mut self,
        wait: Duration,
    ) -> Result<Option<event_bus::ports::EventEnvelope>, ProjectorError> {
        let next = timeout(wait, self.subscriber.next())
            .await
            .map_err(|e| ProjectorError::Source(format!("wait for live event: {e}")))?;

        let Some(message) = next else {
            return Ok(None);
        };

        match serde_json::from_slice::<EventEnvelope>(&message.payload) {
            Ok(envelope) => Ok(Some(envelope)),
            Err(envelope_error) => {
                let event: AppEvent = serde_json::from_slice(&message.payload).map_err(|e| {
                    ProjectorError::Source(format!(
                        "deserialize live envelope: {envelope_error}; app event fallback: {e}"
                    ))
                })?;

                Ok(Some(event_bus::ports::EventEnvelope::new(
                    event,
                    "outbox-relay-worker",
                )))
            }
        }
    }
}
