//! LibSQL implementation of TenantRepository.
//!
//! Translates the abstract TenantRepository trait into concrete SQL operations
//! against a LibSqlPort (embedded Turso / SQLite).

use async_trait::async_trait;
use domain::ports::lib_sql::LibSqlPort;
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

/// LibSQL-backed TenantRepository.
pub struct LibSqlTenantRepository<P: LibSqlPort> {
    port: P,
}

impl<P: LibSqlPort> LibSqlTenantRepository<P> {
    pub fn new(port: P) -> Self {
        Self { port }
    }

    /// Run the tenant table migration (idempotent).
    pub async fn migrate(&self) -> Result<(), RepositoryError> {
        const TENANT_MIGRATION: &str =
            "CREATE TABLE IF NOT EXISTS tenant (\
                id TEXT PRIMARY KEY,\
                name TEXT NOT NULL,\
                created_at TEXT NOT NULL DEFAULT (datetime('now'))\
            )";
        const USER_TENANT_MIGRATION: &str =
            "CREATE TABLE IF NOT EXISTS user_tenant (\
                id TEXT PRIMARY KEY,\
                user_sub TEXT NOT NULL,\
                tenant_id TEXT NOT NULL,\
                role TEXT NOT NULL DEFAULT 'member',\
                FOREIGN KEY (tenant_id) REFERENCES tenant(id)\
            )";
        self.port.execute(TENANT_MIGRATION, vec![]).await?;
        self.port.execute(USER_TENANT_MIGRATION, vec![]).await?;
        Ok(())
    }
}

#[async_trait]
impl<P: LibSqlPort> TenantRepository for LibSqlTenantRepository<P> {
    async fn create_tenant(
        &self,
        input: CreateTenantInput,
    ) -> Result<Tenant, RepositoryError> {
        // SQLite/LibSQL doesn't support RETURNING with generated values reliably.
        // Generate the ID upfront so we know what it is.
        let id: String = uuid::Uuid::new_v4().to_string();

        self.port
            .execute(
                "INSERT INTO tenant (id, name) VALUES (?, ?)",
                vec![id.clone(), input.name.clone()],
            )
            .await?;

        Ok(Tenant {
            id,
            name: input.name,
            created_at: chrono::Utc::now().to_rfc3339(),
        })
    }

    async fn get_tenant(&self, id: &str) -> Result<Option<Tenant>, RepositoryError> {
        let rows: Vec<TenantRow> = self
            .port
            .query(
                "SELECT id, name, created_at FROM tenant WHERE id = ?",
                vec![id.to_string()],
            )
            .await?;
        Ok(rows.into_iter().next().map(row_to_tenant))
    }

    async fn list_tenants(&self) -> Result<Vec<Tenant>, RepositoryError> {
        let rows: Vec<TenantRow> = self
            .port
            .query(
                "SELECT id, name, created_at FROM tenant ORDER BY created_at DESC",
                vec![],
            )
            .await?;
        Ok(rows.into_iter().map(row_to_tenant).collect())
    }

    async fn delete_tenant(&self, id: &str) -> Result<(), RepositoryError> {
        self.port
            .execute("DELETE FROM tenant WHERE id = ?", vec![id.to_string()])
            .await?;
        Ok(())
    }

    async fn find_user_tenant(&self, user_sub: &str) -> Result<Option<UserTenantBinding>, RepositoryError> {
        #[derive(Debug, Deserialize)]
        struct BindingRow {
            tenant_id: String,
            role: String,
        }

        let rows: Vec<BindingRow> = self
            .port
            .query(
                "SELECT tenant_id, role FROM user_tenant WHERE user_sub = ? LIMIT 1",
                vec![user_sub.to_string()],
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
        self.port
            .execute(
                "INSERT INTO user_tenant (id, user_sub, tenant_id, role) VALUES (lower(hex(randomblob(16))), ?, ?, ?)",
                vec![user_sub.to_string(), tenant_id.to_string(), role.to_string()],
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
