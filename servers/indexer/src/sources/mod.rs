//! Event sources — pull events from various protocols.

use async_trait::async_trait;

use crate::IndexerError;

/// Raw event from a protocol source.
#[derive(Debug, Clone)]
pub struct RawEvent {
    pub source: String,       // e.g., "nostr", "farcaster", "evm"
    pub raw_payload: String,   // JSON or protocol-specific format
    pub timestamp: String,
    pub metadata: std::collections::HashMap<String, String>,
}

/// Event source trait — a protocol-specific data puller.
#[async_trait]
pub trait EventSource: Send + Sync {
    /// Name of this source (e.g., "nostr-relay-1").
    fn name(&self) -> &str;

    /// Pull new events since the last cursor.
    async fn pull_events(&self) -> Result<Vec<RawEvent>, IndexerError>;
}

/// In-memory stub source for testing.
pub struct MemoryEventSource {
    pub events: Vec<RawEvent>,
}

impl MemoryEventSource {
    pub fn new(events: Vec<RawEvent>) -> Self {
        Self { events }
    }
}

#[async_trait]
impl EventSource for MemoryEventSource {
    fn name(&self) -> &str {
        "memory-source"
    }

    async fn pull_events(&self) -> Result<Vec<RawEvent>, IndexerError> {
        Ok(self.events.clone())
    }
}
