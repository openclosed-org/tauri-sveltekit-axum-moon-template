//! Shared Turso backend selector for local and remote libSQL paths.

use async_trait::async_trait;
use data_traits::ports::lib_sql::{LibSqlError, LibSqlPort};
use serde::de::DeserializeOwned;

use crate::{EmbeddedTurso, TursoCloud};

/// Selects the appropriate Turso-backed libSQL adapter from configuration.
#[derive(Clone)]
pub enum TursoBackend {
    Embedded(EmbeddedTurso),
    Remote(TursoCloud),
}

impl TursoBackend {
    /// Builds an embedded backend for `file:` or plain local paths, and a remote
    /// backend for `libsql://` URLs that are not local-file aliases.
    pub async fn connect(
        database_url: &str,
        auth_token: Option<&str>,
    ) -> Result<Self, LibSqlError> {
        if is_remote_libsql_url(database_url) {
            let token = auth_token.ok_or_else(|| {
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("missing auth token for remote libsql database '{database_url}'"),
                )) as LibSqlError
            })?;

            Ok(Self::Remote(TursoCloud::new(database_url, token).await?))
        } else {
            Ok(Self::Embedded(EmbeddedTurso::new(database_url).await?))
        }
    }
}

#[async_trait]
impl LibSqlPort for TursoBackend {
    async fn health_check(&self) -> Result<(), LibSqlError> {
        match self {
            Self::Embedded(db) => db.health_check().await,
            Self::Remote(db) => db.health_check().await,
        }
    }

    async fn execute(&self, sql: &str, params: Vec<String>) -> Result<u64, LibSqlError> {
        match self {
            Self::Embedded(db) => db.execute(sql, params).await,
            Self::Remote(db) => db.execute(sql, params).await,
        }
    }

    async fn query<T: DeserializeOwned + Send + Sync>(
        &self,
        sql: &str,
        params: Vec<String>,
    ) -> Result<Vec<T>, LibSqlError> {
        match self {
            Self::Embedded(db) => db.query(sql, params).await,
            Self::Remote(db) => db.query(sql, params).await,
        }
    }
}

fn is_remote_libsql_url(database_url: &str) -> bool {
    database_url.starts_with("libsql://") && !database_url.starts_with("libsql://file:")
}

#[cfg(test)]
mod tests {
    use super::is_remote_libsql_url;

    #[test]
    fn detects_remote_and_local_urls() {
        assert!(is_remote_libsql_url("libsql://counter-db.turso.io"));
        assert!(!is_remote_libsql_url("libsql://file:/tmp/test.db"));
        assert!(!is_remote_libsql_url("file:/tmp/test.db"));
        assert!(!is_remote_libsql_url("/tmp/test.db"));
    }
}
