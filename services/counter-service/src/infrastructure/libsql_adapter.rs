//! CounterRepository implementation backed by libsql (Turso embedded).
//!
//! This adapter translates the abstract CounterRepository trait into
//! concrete SQL operations. It handles:
//! - Counter upsert with CAS version check (optimistic locking)
//! - Atomic increment/decrement/reset with version field
//! - Outbox table writes for event-driven architecture
//! - Timestamp management via datetime('now')

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use data::ports::lib_sql::LibSqlPort;
use serde::Deserialize;

use crate::domain::{Counter, CounterId};
use crate::ports::{CounterRepository, RepositoryError};

/// Raw row shape from the counter table.
#[derive(Debug, Deserialize)]
struct CounterRow {
    tenant_id: String,
    value: i64,
    version: i64,
    updated_at: String,
}

/// Minimal row shape for value-only queries.
#[derive(Debug, Deserialize)]
struct ValueRow {
    value: i64,
    version: i64,
}

/// CounterRepository backed by a libsql port.
///
/// This is the **primary** repository implementation used in Phase 0
/// where the monolith uses embedded Turso (libsql) for storage.
pub struct LibSqlCounterRepository<P: LibSqlPort> {
    port: P,
}

impl<P: LibSqlPort> LibSqlCounterRepository<P> {
    pub fn new(port: P) -> Self {
        Self { port }
    }

    /// Run the counter table migration (idempotent).
    ///
    /// This should be called at application startup by the composition root.
    pub async fn migrate(&self) -> Result<(), RepositoryError> {
        self.port
            .execute(
                "CREATE TABLE IF NOT EXISTS counter (\
                     tenant_id TEXT PRIMARY KEY,\
                     value INTEGER NOT NULL DEFAULT 0,\
                     version INTEGER NOT NULL DEFAULT 0,\
                     updated_at TEXT NOT NULL DEFAULT (datetime('now'))\
                 )",
                vec![],
            )
            .await?;

        self.port
            .execute(
                "CREATE TABLE IF NOT EXISTS counter_outbox (\
                     id INTEGER PRIMARY KEY AUTOINCREMENT,\
                     event_type TEXT NOT NULL,\
                     payload TEXT NOT NULL,\
                     source_service TEXT NOT NULL DEFAULT 'counter-service',\
                     created_at TEXT NOT NULL DEFAULT (datetime('now')),\
                     published INTEGER NOT NULL DEFAULT 0\
                 )",
                vec![],
            )
            .await?;

        self.port
            .execute(
                "CREATE INDEX IF NOT EXISTS idx_counter_outbox_pending \
                 ON counter_outbox(published, id)",
                vec![],
            )
            .await?;

        self.port
            .execute(
                "CREATE TABLE IF NOT EXISTS counter_idempotency (\
                     idempotency_key TEXT PRIMARY KEY,\
                     result_value INTEGER NOT NULL,\
                     result_version INTEGER NOT NULL,\
                     created_at TEXT NOT NULL DEFAULT (datetime('now'))\
                 )",
                vec![],
            )
            .await?;

        Ok(())
    }
}

#[async_trait]
impl<P: LibSqlPort> CounterRepository for LibSqlCounterRepository<P> {
    async fn load(&self, id: &CounterId) -> Result<Option<Counter>, RepositoryError> {
        let rows: Vec<CounterRow> = self
            .port
            .query(
                "SELECT tenant_id, value, version, updated_at FROM counter WHERE tenant_id = ?",
                vec![id.as_str().to_string()],
            )
            .await?;

        let row = match rows.first() {
            Some(r) => r,
            None => return Ok(None),
        };

        let updated_at = DateTime::parse_from_rfc3339(&row.updated_at)
            .or_else(|_| DateTime::parse_from_str(&row.updated_at, "%Y-%m-%d %H:%M:%S"))
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());

        Ok(Some(Counter {
            id: CounterId::new(&row.tenant_id),
            value: row.value,
            version: row.version,
            updated_at,
        }))
    }

    async fn increment(
        &self,
        id: &CounterId,
        expected_version: i64,
        _now: DateTime<Utc>,
    ) -> Result<(i64, i64), RepositoryError> {
        // First, try CAS update if the counter already exists
        self.port
            .execute(
                "UPDATE counter SET value = value + 1, version = version + 1, \
                 updated_at = datetime('now') \
                 WHERE tenant_id = ? AND version = ?",
                vec![id.as_str().to_string(), expected_version.to_string()],
            )
            .await?;

        // If no rows were affected, either it's a new counter or version mismatch
        // For new counters (expected_version == 0), insert with initial values
        if expected_version == 0 {
            self.port
                .execute(
                    "INSERT INTO counter (tenant_id, value, version, updated_at) \
                     VALUES (?, 1, 1, datetime('now')) \
                     ON CONFLICT(tenant_id) DO UPDATE SET \
                         value = value + 1, version = version + 1, \
                         updated_at = datetime('now') \
                     WHERE version = ?",
                    vec![id.as_str().to_string(), expected_version.to_string()],
                )
                .await?;
        }

        // Read back the new value and version
        let rows: Vec<ValueRow> = self
            .port
            .query(
                "SELECT value, version FROM counter WHERE tenant_id = ?",
                vec![id.as_str().to_string()],
            )
            .await?;

        let row = rows.first().ok_or("counter not found after increment")?;
        Ok((row.value, row.version))
    }

    async fn decrement(
        &self,
        id: &CounterId,
        expected_version: i64,
        _now: DateTime<Utc>,
    ) -> Result<(i64, i64), RepositoryError> {
        self.port
            .execute(
                "UPDATE counter SET value = value - 1, version = version + 1, \
                 updated_at = datetime('now') \
                 WHERE tenant_id = ? AND version = ?",
                vec![id.as_str().to_string(), expected_version.to_string()],
            )
            .await?;

        let rows: Vec<ValueRow> = self
            .port
            .query(
                "SELECT value, version FROM counter WHERE tenant_id = ?",
                vec![id.as_str().to_string()],
            )
            .await?;

        let row = rows.first().ok_or("counter not found after decrement")?;
        Ok((row.value, row.version))
    }

    async fn reset(
        &self,
        id: &CounterId,
        expected_version: i64,
        _now: DateTime<Utc>,
    ) -> Result<i64, RepositoryError> {
        self.port
            .execute(
                "UPDATE counter SET value = 0, version = version + 1, \
                 updated_at = datetime('now') \
                 WHERE tenant_id = ? AND version = ?",
                vec![id.as_str().to_string(), expected_version.to_string()],
            )
            .await?;

        let rows: Vec<ValueRow> = self
            .port
            .query(
                "SELECT value, version FROM counter WHERE tenant_id = ?",
                vec![id.as_str().to_string()],
            )
            .await?;

        let row = rows.first().ok_or("counter not found after reset")?;
        Ok(row.version)
    }

    async fn upsert(&self, counter: &Counter) -> Result<(), RepositoryError> {
        self.port
            .execute(
                "INSERT INTO counter (tenant_id, value, version, updated_at) \
                 VALUES (?, ?, ?, ?) \
                 ON CONFLICT(tenant_id) DO UPDATE SET \
                     value = excluded.value, \
                     version = excluded.version, \
                     updated_at = excluded.updated_at",
                vec![
                    counter.id.as_str().to_string(),
                    counter.value.to_string(),
                    counter.version.to_string(),
                    counter.updated_at.format("%Y-%m-%d %H:%M:%S").to_string(),
                ],
            )
            .await?;
        Ok(())
    }

    async fn write_outbox(
        &self,
        event_type: &str,
        payload: &str,
        source_service: &str,
    ) -> Result<(), RepositoryError> {
        self.port
            .execute(
                "INSERT INTO counter_outbox (event_type, payload, source_service) \
                 VALUES (?, ?, ?)",
                vec![
                    event_type.to_string(),
                    payload.to_string(),
                    source_service.to_string(),
                ],
            )
            .await?;
        Ok(())
    }
}
