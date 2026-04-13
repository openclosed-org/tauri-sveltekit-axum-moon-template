//! SurrealDB implementation of TenantRepository.
//!
//! Translates the abstract TenantRepository trait into concrete SurrealQL operations.

use std::collections::BTreeMap;

use async_trait::async_trait;
use domain::ports::surreal_db::SurrealDbPort;
use serde::Deserialize;

use crate::domain::{CreateTenantInput, Tenant};
use crate::ports::{RepositoryError, TenantRepository, UserTenantBinding};

/// Raw row shape from the tenant table.
#[derive(Debug, Deserialize)]
struct TenantRow {
    id: String,
    name: String,
    created_at: String,
}

/// SurrealDB-backed TenantRepository.
pub struct SurrealDbTenantRepository<P: SurrealDbPort> {
    port: P,
}

impl<P: SurrealDbPort> SurrealDbTenantRepository<P> {
    pub fn new(port: P) -> Self {
        Self { port }
    }
}

#[async_trait]
impl<P: SurrealDbPort> TenantRepository for SurrealDbTenantRepository<P> {
    async fn create_tenant(
        &self,
        input: CreateTenantInput,
    ) -> Result<Tenant, RepositoryError> {
        let mut vars = BTreeMap::new();
        vars.insert("id".into(), serde_json::json!(input.id));
        vars.insert("name".into(), serde_json::json!(input.name));

        let rows: Vec<TenantRow> = self
            .port
            .query(
                "CREATE tenant CONTENT { id: $id, name: $name, created_at: time::now() } RETURN AFTER",
                vars,
            )
            .await?;

        rows.into_iter()
            .next()
            .map(row_to_tenant)
            .ok_or_else(|| RepositoryError::from("Failed to create tenant"))
    }

    async fn get_tenant(&self, id: &str) -> Result<Option<Tenant>, RepositoryError> {
        let mut vars = BTreeMap::new();
        vars.insert("id".into(), serde_json::json!(id));

        let rows: Vec<TenantRow> = self
            .port
            .query("SELECT * FROM tenant WHERE id = $id", vars)
            .await?;

        Ok(rows.into_iter().next().map(row_to_tenant))
    }

    async fn list_tenants(&self) -> Result<Vec<Tenant>, RepositoryError> {
        let rows: Vec<TenantRow> = self
            .port
            .query(
                "SELECT * FROM tenant ORDER BY created_at DESC",
                BTreeMap::new(),
            )
            .await?;
        Ok(rows.into_iter().map(row_to_tenant).collect())
    }

    async fn delete_tenant(&self, id: &str) -> Result<(), RepositoryError> {
        let mut vars = BTreeMap::new();
        vars.insert("id".into(), serde_json::json!(id));

        let _: Vec<serde_json::Value> = self
            .port
            .query("DELETE tenant WHERE id = $id", vars)
            .await?;

        Ok(())
    }

    async fn find_user_tenant(&self, user_sub: &str) -> Result<Option<UserTenantBinding>, RepositoryError> {
        #[derive(Debug, Deserialize)]
        struct BindingRow {
            tenant_id: String,
            role: String,
        }

        let mut vars = BTreeMap::new();
        vars.insert("user_sub".into(), serde_json::json!(user_sub));

        let rows: Vec<BindingRow> = self
            .port
            .query(
                "SELECT tenant_id, role FROM user_tenant WHERE user_sub = $user_sub LIMIT 1",
                vars,
            )
            .await?;

        Ok(rows.into_iter().next().map(|row| UserTenantBinding {
            tenant_id: row.tenant_id,
            role: row.role,
        }))
    }

    async fn create_user_tenant_binding(
        &self,
        user_sub: &str,
        tenant_id: &str,
        role: &str,
    ) -> Result<(), RepositoryError> {
        let mut vars = BTreeMap::new();
        vars.insert("user_sub".into(), serde_json::json!(user_sub));
        vars.insert("tenant_id".into(), serde_json::json!(tenant_id));
        vars.insert("role".into(), serde_json::json!(role));

        let _: Vec<serde_json::Value> = self
            .port
            .query(
                "CREATE user_tenant CONTENT { user_sub: $user_sub, tenant_id: $tenant_id, role: $role }",
                vars,
            )
            .await?;

        Ok(())
    }
}

fn row_to_tenant(row: TenantRow) -> Tenant {
    Tenant {
        id: row.id,
        name: row.name,
        created_at: row.created_at,
    }
}
