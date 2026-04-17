//! CounterRepository implementation backed by libsql (Turso embedded).
//!
//! This adapter translates the abstract CounterRepository trait into
//! concrete SQL operations. It handles:
//! - Counter upsert with CAS version check (optimistic locking)
//! - Atomic increment/decrement/reset with version field
//! - Unified event_outbox writes (no per-service private outbox)
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

/// Minimal row shape for value-only queries from counter table.
#[derive(Debug, Deserialize)]
struct ValueRow {
    value: i64,
    version: i64,
}

/// Row shape from the counter_idempotency table.
#[derive(Debug, Deserialize)]
struct IdempotencyRow {
    result_value: i64,
    result_version: i64,
}

/// CounterRepository backed by a libsql port.
///
/// This is the **primary** repository implementation used in Phase 0
/// where the monolith uses embedded Turso (libsql) for storage.
pub struct LibSqlCounterRepository<P: LibSqlPort> {
    pub port: P,
}

impl<P: LibSqlPort> LibSqlCounterRepository<P> {
    pub fn new(port: P) -> Self {
        Self { port }
    }

    /// Run the counter table migrations (idempotent).
    ///
    /// This should be called at application startup by the composition root.
    /// SQL schema is loaded from the migration file at `migrations/001_create_counter.sql`
    /// to maintain a single source of truth for the database schema.
    pub async fn migrate(&self) -> Result<(), RepositoryError> {
        let migration_sql = include_str!("../../migrations/001_create_counter.sql");

        // Split by semicolons and execute each statement
        for statement in migration_sql.split(';') {
            // Remove comment lines and whitespace
            let cleaned: String = statement
                .lines()
                .filter(|line| !line.trim().starts_with("--"))
                .collect::<Vec<_>>()
                .join("\n");
            let trimmed = cleaned.trim();
            if trimmed.is_empty() {
                continue;
            }
            self.port.execute(trimmed, vec![]).await?;
        }

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
        if expected_version == 0 {
            // New counter: insert with initial values
            let rows_affected = self
                .port
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

            if rows_affected == 0 {
                // Conflict: counter exists but version mismatch
                return Err("CAS conflict on increment".into());
            }
        } else {
            // Existing counter: CAS update
            let rows_affected = self
                .port
                .execute(
                    "UPDATE counter SET value = value + 1, version = version + 1, \
                     updated_at = datetime('now') \
                     WHERE tenant_id = ? AND version = ?",
                    vec![id.as_str().to_string(), expected_version.to_string()],
                )
                .await?;

            if rows_affected == 0 {
                return Err("CAS conflict on increment".into());
            }
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
        let rows_affected = self
            .port
            .execute(
                "UPDATE counter SET value = value - 1, version = version + 1, \
                 updated_at = datetime('now') \
                 WHERE tenant_id = ? AND version = ?",
                vec![id.as_str().to_string(), expected_version.to_string()],
            )
            .await?;

        if rows_affected == 0 {
            return Err("CAS conflict on decrement".into());
        }

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
        let rows_affected = self
            .port
            .execute(
                "UPDATE counter SET value = 0, version = version + 1, \
                 updated_at = datetime('now') \
                 WHERE tenant_id = ? AND version = ?",
                vec![id.as_str().to_string(), expected_version.to_string()],
            )
            .await?;

        if rows_affected == 0 {
            return Err("CAS conflict on reset".into());
        }

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
        event_id: &str,
        event_type: &str,
        payload: &str,
        source_service: &str,
        correlation_id: Option<&str>,
    ) -> Result<(), RepositoryError> {
        self.port
            .execute(
                "INSERT INTO event_outbox (event_id, event_type, event_payload, source_service, correlation_id, status) \
                 VALUES (?, ?, ?, ?, ?, 'pending')",
                vec![
                    event_id.to_string(),
                    event_type.to_string(),
                    payload.to_string(),
                    source_service.to_string(),
                    correlation_id.unwrap_or_default().to_string(),
                ],
            )
            .await?;
        Ok(())
    }

    async fn check_idempotency(&self, key: &str) -> Result<Option<(i64, i64)>, RepositoryError> {
        let rows: Vec<IdempotencyRow> = self
            .port
            .query(
                "SELECT result_value, result_version FROM counter_idempotency \
                 WHERE idempotency_key = ?",
                vec![key.to_string()],
            )
            .await?;

        match rows.first() {
            Some(r) => Ok(Some((r.result_value, r.result_version))),
            None => Ok(None),
        }
    }

    async fn cache_idempotency(
        &self,
        key: &str,
        value: i64,
        version: i64,
    ) -> Result<(), RepositoryError> {
        self.port
            .execute(
                "INSERT INTO counter_idempotency (idempotency_key, result_value, result_version) \
                 VALUES (?, ?, ?) \
                 ON CONFLICT(idempotency_key) DO NOTHING",
                vec![key.to_string(), value.to_string(), version.to_string()],
            )
            .await?;
        Ok(())
    }
}
