//! LibSQL implementation of SessionRepository.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::sync::Arc;
use turso::{Builder, Database, Value};

use crate::domain::error::AuthError;
use crate::domain::session::Session;
use crate::ports::SessionRepository;

/// LibSQL-based session repository.
#[derive(Clone)]
pub struct LibSqlSessionRepository {
    db: Arc<Database>,
}

impl LibSqlSessionRepository {
    /// Create a new session repository from an existing Turso Database.
    pub fn new(db: Database) -> Self {
        Self {
            db: Arc::new(db),
        }
    }

    /// Create a new session repository from a local database path.
    pub async fn from_path(path: &str) -> Result<Self, AuthError> {
        let db = Builder::new_local(path)
            .build()
            .await
            .map_err(|e| AuthError::Database(format!("Failed to create database: {e}")))?;
        Ok(Self::new(db))
    }

    /// Initialize the sessions table if it doesn't exist.
    pub async fn initialize(&self) -> Result<(), AuthError> {
        let conn = self
            .db
            .connect()
            .map_err(|e| AuthError::Database(format!("Failed to connect to database: {e}")))?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                user_id TEXT NOT NULL,
                user_sub TEXT NOT NULL,
                tenant_id TEXT,
                expires_at TEXT NOT NULL,
                created_at TEXT NOT NULL,
                last_accessed_at TEXT NOT NULL,
                ip_address TEXT,
                user_agent TEXT
            )",
            (),
        )
        .await
        .map_err(|e| AuthError::Database(format!("Failed to create sessions table: {e}")))?;

        Ok(())
    }

    /// Convert a turso::Value to String.
    fn value_to_string(value: &Value) -> Result<String, AuthError> {
        match value {
            Value::Text(s) => Ok(s.clone()),
            Value::Null => Ok(String::new()),
            other => Err(AuthError::Database(format!(
                "Expected text value, got {other:?}"
            ))),
        }
    }

    /// Extract a session from a result row.
    fn extract_session(row: &turso::Row) -> Result<Session, AuthError> {
        let id = Self::value_to_string(&row.get_value(0).map_err(|e| {
            AuthError::Database(format!("Failed to get id: {e}"))
        })?)?;
        let user_id = Self::value_to_string(&row.get_value(1).map_err(|e| {
            AuthError::Database(format!("Failed to get user_id: {e}"))
        })?)?;
        let user_sub = Self::value_to_string(&row.get_value(2).map_err(|e| {
            AuthError::Database(format!("Failed to get user_sub: {e}"))
        })?)?;
        let tenant_id = match row.get_value(3).map_err(|e| {
            AuthError::Database(format!("Failed to get tenant_id: {e}"))
        })? {
            Value::Text(s) => Some(s),
            Value::Null => None,
            other => {
                return Err(AuthError::Database(format!(
                    "Invalid tenant_id value: {other:?}"
                )))
            }
        };
        let expires_at_str = Self::value_to_string(&row.get_value(4).map_err(|e| {
            AuthError::Database(format!("Failed to get expires_at: {e}"))
        })?)?;
        let created_at_str = Self::value_to_string(&row.get_value(5).map_err(|e| {
            AuthError::Database(format!("Failed to get created_at: {e}"))
        })?)?;
        let last_accessed_at_str = Self::value_to_string(&row.get_value(6).map_err(|e| {
            AuthError::Database(format!("Failed to get last_accessed_at: {e}"))
        })?)?;
        let ip_address = match row.get_value(7).map_err(|e| {
            AuthError::Database(format!("Failed to get ip_address: {e}"))
        })? {
            Value::Text(s) => Some(s),
            Value::Null => None,
            other => {
                return Err(AuthError::Database(format!(
                    "Invalid ip_address value: {other:?}"
                )))
            }
        };
        let user_agent = match row.get_value(8).map_err(|e| {
            AuthError::Database(format!("Failed to get user_agent: {e}"))
        })? {
            Value::Text(s) => Some(s),
            Value::Null => None,
            other => {
                return Err(AuthError::Database(format!(
                    "Invalid user_agent value: {other:?}"
                )))
            }
        };

        let expires_at = DateTime::parse_from_rfc3339(&expires_at_str)
            .map_err(|e| AuthError::Database(format!("Invalid expires_at: {e}")))?
            .into();
        let created_at = DateTime::parse_from_rfc3339(&created_at_str)
            .map_err(|e| AuthError::Database(format!("Invalid created_at: {e}")))?
            .into();
        let last_accessed_at = DateTime::parse_from_rfc3339(&last_accessed_at_str)
            .map_err(|e| AuthError::Database(format!("Invalid last_accessed_at: {e}")))?
            .into();

        Ok(Session {
            id,
            user_id,
            user_sub,
            tenant_id,
            expires_at,
            created_at,
            last_accessed_at,
            ip_address,
            user_agent,
        })
    }
}

#[async_trait]
impl SessionRepository for LibSqlSessionRepository {
    async fn create_session(&self, session: &Session) -> Result<(), AuthError> {
        let conn = self
            .db
            .connect()
            .map_err(|e| AuthError::Database(format!("Failed to connect to database: {e}")))?;

        conn.execute(
            "INSERT INTO sessions (id, user_id, user_sub, tenant_id, expires_at, created_at, last_accessed_at, ip_address, user_agent)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            (
                Value::Text(session.id.clone()),
                Value::Text(session.user_id.clone()),
                Value::Text(session.user_sub.clone()),
                session.tenant_id.as_ref().map_or(Value::Null, |s| Value::Text(s.clone())),
                Value::Text(session.expires_at.to_rfc3339()),
                Value::Text(session.created_at.to_rfc3339()),
                Value::Text(session.last_accessed_at.to_rfc3339()),
                session.ip_address.as_ref().map_or(Value::Null, |s| Value::Text(s.clone())),
                session.user_agent.as_ref().map_or(Value::Null, |s| Value::Text(s.clone())),
            ),
        )
        .await
        .map_err(|e| AuthError::Database(format!("Failed to create session: {e}")))?;

        Ok(())
    }

    async fn get_session(&self, session_id: &str) -> Result<Option<Session>, AuthError> {
        let conn = self
            .db
            .connect()
            .map_err(|e| AuthError::Database(format!("Failed to connect to database: {e}")))?;

        let mut rows = conn
            .query(
                "SELECT id, user_id, user_sub, tenant_id, expires_at, created_at, last_accessed_at, ip_address, user_agent
                 FROM sessions
                 WHERE id = ?1",
                (Value::Text(session_id.to_string()),),
            )
            .await
            .map_err(|e| AuthError::Database(format!("Failed to query session: {e}")))?;

        if let Some(row) = rows
            .next()
            .await
            .map_err(|e| AuthError::Database(format!("Failed to fetch session row: {e}")))?
        {
            let session = Self::extract_session(&row)?;
            Ok(Some(session))
        } else {
            Ok(None)
        }
    }

    async fn delete_session(&self, session_id: &str) -> Result<(), AuthError> {
        let conn = self
            .db
            .connect()
            .map_err(|e| AuthError::Database(format!("Failed to connect to database: {e}")))?;

        conn.execute(
            "DELETE FROM sessions WHERE id = ?1",
            (Value::Text(session_id.to_string()),),
        )
        .await
        .map_err(|e| AuthError::Database(format!("Failed to delete session: {e}")))?;

        Ok(())
    }

    async fn touch_session(&self, session_id: &str) -> Result<(), AuthError> {
        let conn = self
            .db
            .connect()
            .map_err(|e| AuthError::Database(format!("Failed to connect to database: {e}")))?;

        let now = Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE sessions SET last_accessed_at = ?1 WHERE id = ?2",
            (Value::Text(now), Value::Text(session_id.to_string())),
        )
        .await
        .map_err(|e| AuthError::Database(format!("Failed to update session: {e}")))?;

        Ok(())
    }
}
