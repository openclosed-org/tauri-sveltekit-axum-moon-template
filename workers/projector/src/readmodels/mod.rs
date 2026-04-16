//! Read model builders — materialized views built from event streams.

use async_trait::async_trait;
use data::ports::lib_sql::LibSqlPort;

use crate::consumers::CounterStateUpdate;

use crate::ProjectorError;

/// A read model — a materialized view updated from events.
#[async_trait]
pub trait ReadModel: Send + Sync {
    /// Name of this read model.
    fn name(&self) -> &str;

    /// Apply an update from a consumer.
    async fn apply_update(&self, update: &str) -> Result<(), ProjectorError>;
}

/// In-memory stub read model for testing.
pub struct MemoryReadModel {
    pub updates: tokio::sync::Mutex<Vec<String>>,
}

impl MemoryReadModel {
    pub fn new() -> Self {
        Self {
            updates: tokio::sync::Mutex::new(Vec::new()),
        }
    }
}

impl Default for MemoryReadModel {
    fn default() -> Self {
        Self::new()
    }
}

pub struct SqliteCounterReadModel<P: LibSqlPort> {
    port: P,
}

impl<P: LibSqlPort> SqliteCounterReadModel<P> {
    pub fn new(port: P) -> Self {
        Self { port }
    }

    pub async fn init(&self) -> Result<(), ProjectorError> {
        self.port
            .execute(
                "CREATE TABLE IF NOT EXISTS counter_projection (\
                    tenant_id TEXT NOT NULL,\
                    counter_key TEXT NOT NULL,\
                    value INTEGER NOT NULL,\
                    version INTEGER NOT NULL,\
                    operation TEXT NOT NULL,\
                    projected_at TEXT NOT NULL,\
                    PRIMARY KEY (tenant_id, counter_key)\
                )",
                vec![],
            )
            .await
            .map_err(|e| ProjectorError::ReadModel(format!("init counter_projection: {e}")))?;
        Ok(())
    }
}

#[async_trait]
impl<P: LibSqlPort> ReadModel for SqliteCounterReadModel<P> {
    fn name(&self) -> &str {
        "sqlite-counter-read-model"
    }

    async fn apply_update(&self, update: &str) -> Result<(), ProjectorError> {
        let update: CounterStateUpdate = serde_json::from_str(update)
            .map_err(|e| ProjectorError::ReadModel(format!("deserialize update: {e}")))?;

        self.port
            .execute(
                "INSERT INTO counter_projection (tenant_id, counter_key, value, version, operation, projected_at) \
                 VALUES (?, ?, ?, ?, ?, ?) \
                 ON CONFLICT(tenant_id, counter_key) DO UPDATE SET \
                    value = excluded.value, \
                    version = excluded.version, \
                    operation = excluded.operation, \
                    projected_at = excluded.projected_at",
                vec![
                    update.tenant_id,
                    update.counter_key,
                    update.new_value.to_string(),
                    update.version.to_string(),
                    update.operation,
                    update.projected_at,
                ],
            )
            .await
            .map_err(|e| ProjectorError::ReadModel(format!("upsert counter_projection: {e}")))?;

        Ok(())
    }
}

#[async_trait]
impl ReadModel for MemoryReadModel {
    fn name(&self) -> &str {
        "memory-read-model"
    }

    async fn apply_update(&self, update: &str) -> Result<(), ProjectorError> {
        self.updates.lock().await.push(update.to_string());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn read_model_collects_updates() {
        let model = MemoryReadModel::new();
        model.apply_update("update-1").await.unwrap();
        model.apply_update("update-2").await.unwrap();

        let updates = model.updates.lock().await;
        assert_eq!(updates.len(), 2);
    }
}
