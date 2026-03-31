//! Tenant service — manages tenant CRUD operations.
//!
//! Uses LibSqlPort for local, SurrealDbPort/TursoPort for cloud.
//! Provides: create_tenant, get_tenant, list_tenants, delete_tenant.

use async_trait::async_trait;
use domain::ports::lib_sql::LibSqlPort;
use domain::ports::surreal_db::SurrealDbPort;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Tenant entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tenant {
    pub id: String,
    pub name: String,
    pub created_at: String,
}

/// Input for creating a new tenant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTenantInput {
    pub id: String,
    pub name: String,
}

/// Error type for tenant service operations.
#[derive(Debug, thiserror::Error)]
pub enum TenantServiceError {
    #[error("Database error: {0}")]
    Database(#[from] Box<dyn std::error::Error + Send + Sync>),
    #[error("Tenant not found: {0}")]
    NotFound(String),
    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

/// Trait for tenant operations — abstracts over database backend.
#[async_trait]
pub trait TenantService: Send + Sync {
    async fn create_tenant(&self, input: CreateTenantInput) -> Result<Tenant, TenantServiceError>;
    async fn get_tenant(&self, id: &str) -> Result<Option<Tenant>, TenantServiceError>;
    async fn list_tenants(&self) -> Result<Vec<Tenant>, TenantServiceError>;
    async fn delete_tenant(&self, id: &str) -> Result<(), TenantServiceError>;
}

/// TenantService backed by LibSqlPort (local/embedded).
pub struct LibSqlTenantService<P: LibSqlPort> {
    port: P,
}

impl<P: LibSqlPort> LibSqlTenantService<P> {
    pub fn new(port: P) -> Self {
        Self { port }
    }
}

#[async_trait]
impl<P: LibSqlPort> TenantService for LibSqlTenantService<P> {
    async fn create_tenant(&self, input: CreateTenantInput) -> Result<Tenant, TenantServiceError> {
        self.port
            .execute(
                "INSERT INTO tenant (id, name) VALUES (?, ?)",
                vec![input.id.clone(), input.name.clone()],
            )
            .await
            .map_err(TenantServiceError::Database)?;

        Ok(Tenant {
            id: input.id,
            name: input.name,
            created_at: chrono::Utc::now().to_rfc3339(),
        })
    }

    async fn get_tenant(&self, id: &str) -> Result<Option<Tenant>, TenantServiceError> {
        let tenants: Vec<Tenant> = self
            .port
            .query(
                "SELECT id, name, created_at FROM tenant WHERE id = ?",
                vec![id.to_string()],
            )
            .await
            .map_err(TenantServiceError::Database)?;
        Ok(tenants.into_iter().next())
    }

    async fn list_tenants(&self) -> Result<Vec<Tenant>, TenantServiceError> {
        self.port
            .query(
                "SELECT id, name, created_at FROM tenant ORDER BY created_at DESC",
                vec![],
            )
            .await
            .map_err(TenantServiceError::Database)
    }

    async fn delete_tenant(&self, id: &str) -> Result<(), TenantServiceError> {
        self.port
            .execute("DELETE FROM tenant WHERE id = ?", vec![id.to_string()])
            .await
            .map_err(TenantServiceError::Database)?;
        Ok(())
    }
}

/// TenantService backed by SurrealDbPort (cloud).
pub struct SurrealDbTenantService<P: SurrealDbPort> {
    port: P,
}

impl<P: SurrealDbPort> SurrealDbTenantService<P> {
    pub fn new(port: P) -> Self {
        Self { port }
    }
}

#[async_trait]
impl<P: SurrealDbPort> TenantService for SurrealDbTenantService<P> {
    async fn create_tenant(&self, input: CreateTenantInput) -> Result<Tenant, TenantServiceError> {
        let mut vars = BTreeMap::new();
        vars.insert("id".into(), serde_json::json!(input.id));
        vars.insert("name".into(), serde_json::json!(input.name));

        let tenants: Vec<Tenant> = self
            .port
            .query(
                "CREATE tenant CONTENT { id: $id, name: $name, created_at: time::now() } RETURN AFTER",
                vars,
            )
            .await
            .map_err(TenantServiceError::Database)?;

        tenants
            .into_iter()
            .next()
            .ok_or_else(|| TenantServiceError::NotFound("Failed to create tenant".into()))
    }

    async fn get_tenant(&self, id: &str) -> Result<Option<Tenant>, TenantServiceError> {
        let mut vars = BTreeMap::new();
        vars.insert("id".into(), serde_json::json!(id));

        let tenants: Vec<Tenant> = self
            .port
            .query("SELECT * FROM tenant WHERE id = $id", vars)
            .await
            .map_err(TenantServiceError::Database)?;

        Ok(tenants.into_iter().next())
    }

    async fn list_tenants(&self) -> Result<Vec<Tenant>, TenantServiceError> {
        self.port
            .query(
                "SELECT * FROM tenant ORDER BY created_at DESC",
                BTreeMap::new(),
            )
            .await
            .map_err(TenantServiceError::Database)
    }

    async fn delete_tenant(&self, id: &str) -> Result<(), TenantServiceError> {
        let mut vars = BTreeMap::new();
        vars.insert("id".into(), serde_json::json!(id));

        let _: Vec<serde_json::Value> = self
            .port
            .query("DELETE tenant WHERE id = $id", vars)
            .await
            .map_err(TenantServiceError::Database)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::de::DeserializeOwned;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    /// Mock LibSqlPort for testing.
    struct MockLibSqlPort {
        tenants: Arc<Mutex<Vec<Tenant>>>,
    }

    impl MockLibSqlPort {
        fn new() -> Self {
            Self {
                tenants: Arc::new(Mutex::new(vec![])),
            }
        }
    }

    #[async_trait]
    impl LibSqlPort for MockLibSqlPort {
        async fn health_check(&self) -> Result<(), domain::ports::lib_sql::LibSqlError> {
            Ok(())
        }

        async fn execute(
            &self,
            _sql: &str,
            params: Vec<String>,
        ) -> Result<u64, domain::ports::lib_sql::LibSqlError> {
            if params.len() == 2 {
                let mut tenants = self.tenants.lock().await;
                tenants.push(Tenant {
                    id: params[0].clone(),
                    name: params[1].clone(),
                    created_at: chrono::Utc::now().to_rfc3339(),
                });
                Ok(1)
            } else if params.len() == 1 {
                let mut tenants = self.tenants.lock().await;
                let before = tenants.len();
                tenants.retain(|t| t.id != params[0]);
                Ok((before - tenants.len()) as u64)
            } else {
                Ok(0)
            }
        }

        async fn query<T: DeserializeOwned + Send + Sync>(
            &self,
            _sql: &str,
            params: Vec<String>,
        ) -> Result<Vec<T>, domain::ports::lib_sql::LibSqlError> {
            let tenants = self.tenants.lock().await;
            if params.is_empty() {
                let json = serde_json::to_value(&*tenants).unwrap();
                let items: Vec<T> = serde_json::from_value(json)
                    .map_err(|e| Box::new(e) as domain::ports::lib_sql::LibSqlError)?;
                Ok(items)
            } else {
                let filtered: Vec<&Tenant> = tenants.iter().filter(|t| t.id == params[0]).collect();
                let json = serde_json::to_value(&filtered).unwrap();
                let items: Vec<T> = serde_json::from_value(json)
                    .map_err(|e| Box::new(e) as domain::ports::lib_sql::LibSqlError)?;
                Ok(items)
            }
        }
    }

    #[tokio::test]
    async fn test_create_and_get_tenant() {
        let port = MockLibSqlPort::new();
        let service = LibSqlTenantService::new(port);

        let input = CreateTenantInput {
            id: "tenant-1".into(),
            name: "Test Tenant".into(),
        };

        let created = service.create_tenant(input.clone()).await.unwrap();
        assert_eq!(created.id, "tenant-1");
        assert_eq!(created.name, "Test Tenant");

        let found = service.get_tenant("tenant-1").await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "Test Tenant");
    }

    #[tokio::test]
    async fn test_list_tenants() {
        let port = MockLibSqlPort::new();
        let service = LibSqlTenantService::new(port);

        service
            .create_tenant(CreateTenantInput {
                id: "t1".into(),
                name: "Tenant 1".into(),
            })
            .await
            .unwrap();

        service
            .create_tenant(CreateTenantInput {
                id: "t2".into(),
                name: "Tenant 2".into(),
            })
            .await
            .unwrap();

        let tenants = service.list_tenants().await.unwrap();
        assert_eq!(tenants.len(), 2);
    }

    #[tokio::test]
    async fn test_delete_tenant() {
        let port = MockLibSqlPort::new();
        let service = LibSqlTenantService::new(port);

        service
            .create_tenant(CreateTenantInput {
                id: "t1".into(),
                name: "To Delete".into(),
            })
            .await
            .unwrap();

        service.delete_tenant("t1").await.unwrap();

        let found = service.get_tenant("t1").await.unwrap();
        assert!(found.is_none());
    }

    #[tokio::test]
    async fn test_get_nonexistent_tenant() {
        let port = MockLibSqlPort::new();
        let service = LibSqlTenantService::new(port);

        let found = service.get_tenant("nonexistent").await.unwrap();
        assert!(found.is_none());
    }
}
