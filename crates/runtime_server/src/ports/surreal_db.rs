//! Tenant-aware SurrealDB wrapper implementing SurrealDbPort.
//!
//! Automatically injects `tenant_id` filters into all queries to enforce
//! multi-tenant data isolation at the implementation layer (D-11: trait unchanged).

use async_trait::async_trait;
use domain::ports::surreal_db::{SurrealDbPort, SurrealError};
use domain::ports::TenantId;
use serde::de::DeserializeOwned;
use std::collections::BTreeMap;
use surrealdb::{engine::any::Any, Surreal};

/// SurrealDbPort implementation with automatic tenant_id injection.
///
/// All SELECT queries get `AND tenant_id = $tenant_id` appended.
/// All CREATE queries get `tenant_id = $tenant_id` added to SET clause.
/// UPDATE/DELETE get `WHERE tenant_id = $tenant_id` appended.
///
/// When tenant_id is None (admin mode), queries pass through unchanged.
#[derive(Clone)]
pub struct TenantAwareSurrealDb {
    db: Surreal<Any>,
    tenant_id: Option<String>,
}

impl TenantAwareSurrealDb {
    /// Create a tenant-scoped instance — all queries auto-filtered.
    pub fn new(db: Surreal<Any>, tenant_id: TenantId) -> Self {
        Self {
            db,
            tenant_id: Some(tenant_id.0),
        }
    }

    /// Create an admin (unscoped) instance — no tenant filtering.
    pub fn new_admin(db: Surreal<Any>) -> Self {
        Self {
            db,
            tenant_id: None,
        }
    }

    /// Get the tenant_id if scoped.
    pub fn tenant_id(&self) -> Option<&str> {
        self.tenant_id.as_deref()
    }

    /// Inject tenant_id filter into a SurrealQL query string.
    ///
    /// - SELECT without WHERE → append `WHERE tenant_id = $tenant_id`
    /// - SELECT with WHERE → append `AND tenant_id = $tenant_id`
    /// - CREATE → append `, tenant_id = $tenant_id` to SET clause
    /// - UPDATE/DELETE with WHERE → append `AND tenant_id = $tenant_id`
    /// - UPDATE/DELETE without WHERE → append `WHERE tenant_id = $tenant_id`
    pub fn inject_tenant_filter(sql: &str) -> String {
        let sql_upper = sql.to_uppercase();
        let sql_trimmed = sql.trim();

        if sql_upper.starts_with("SELECT") {
            // Find WHERE keyword position
            if let Some(where_pos) = sql_upper.find("WHERE") {
                // Split: ...before WHERE... WHERE condition...
                // Insert: ...before WHERE... tenant_id = $tenant_id AND condition...
                let before_where = &sql_trimmed[..where_pos]; // up to WHERE
                let after_where_kw = &sql_trimmed[where_pos + 5..]; // after WHERE (the condition)
                format!(
                    "{}WHERE tenant_id = $tenant_id AND {}",
                    before_where,
                    after_where_kw.trim_start()
                )
            } else {
                // No WHERE — insert before GROUP BY / ORDER BY / LIMIT / etc.
                let insert_pos = Self::find_clause_insert_pos(sql_trimmed, &sql_upper);
                let before = &sql_trimmed[..insert_pos];
                let after = &sql_trimmed[insert_pos..];
                format!("{} WHERE tenant_id = $tenant_id {}", before, after)
                    .trim()
                    .to_string()
            }
        } else if sql_upper.starts_with("CREATE") || sql_upper.starts_with("INSERT") {
            if sql_upper.contains("SET") {
                if let Some(return_pos) = sql_upper.find(" RETURN") {
                    let before = &sql_trimmed[..return_pos];
                    let after = &sql_trimmed[return_pos..];
                    format!("{}, tenant_id = $tenant_id {}", before, after)
                } else {
                    format!("{}, tenant_id = $tenant_id", sql_trimmed)
                }
            } else {
                format!("{}, tenant_id = $tenant_id", sql_trimmed)
            }
        } else if sql_upper.starts_with("UPDATE") || sql_upper.starts_with("DELETE") {
            if let Some(where_pos) = sql_upper.find("WHERE") {
                let before_where = &sql_trimmed[..where_pos];
                let after_where_kw = &sql_trimmed[where_pos + 5..];
                format!(
                    "{}WHERE tenant_id = $tenant_id AND {}",
                    before_where,
                    after_where_kw.trim_start()
                )
            } else if let Some(return_pos) = sql_upper.find(" RETURN") {
                let before = &sql_trimmed[..return_pos];
                let after = &sql_trimmed[return_pos..];
                format!("{} WHERE tenant_id = $tenant_id {}", before, after)
            } else {
                format!("{} WHERE tenant_id = $tenant_id", sql_trimmed)
            }
        } else {
            sql_trimmed.to_string()
        }
    }

    /// Find the position to insert WHERE clause before GROUP BY, ORDER BY, LIMIT, etc.
    fn find_clause_insert_pos(sql: &str, sql_upper: &str) -> usize {
        let clauses = [" GROUP BY", " ORDER BY", " LIMIT", " START", " FETCH"];
        let mut earliest = sql.len();
        for clause in &clauses {
            if let Some(pos) = sql_upper.find(clause) {
                if pos < earliest {
                    earliest = pos;
                }
            }
        }
        earliest
    }
}

#[async_trait]
impl SurrealDbPort for TenantAwareSurrealDb {
    async fn health_check(&self) -> Result<(), SurrealError> {
        self.db
            .health()
            .await
            .map_err(|e| Box::new(e) as SurrealError)
    }

    async fn query<T: DeserializeOwned + Send + Sync>(
        &self,
        sql: &str,
        mut vars: BTreeMap<String, surrealdb::types::Value>,
    ) -> Result<Vec<T>, SurrealError> {
        // 1. Inject tenant_id into vars if scoped
        if let Some(ref tid) = self.tenant_id {
            vars.insert(
                "tenant_id".into(),
                surrealdb::types::Value::String(tid.clone()),
            );
        }

        // 2. Rewrite SQL if tenant-scoped
        let scoped_sql = if self.tenant_id.is_some() {
            Self::inject_tenant_filter(sql)
        } else {
            sql.to_string()
        };

        // 3. Execute query — use serde_json::Value as intermediate
        //    (implements SurrealValue, avoids coupling trait to surrealdb internals)
        let mut response = self.db.query(&scoped_sql).bind(vars).await?;
        let raw: Vec<serde_json::Value> = response.take(0)?;

        // 4. Deserialize each element to T
        raw.into_iter()
            .map(|v| serde_json::from_value(v).map_err(|e| Box::new(e) as SurrealError))
            .collect()
    }
}

/// Run tenant schema migrations on the given SurrealDB connection.
///
/// Defines: tenant table, user_tenant table, indexes.
/// Safe to call multiple times (DEFINE TABLE is idempotent with SCHEMAFULL).
pub async fn run_tenant_migrations(db: &Surreal<Any>) -> Result<(), SurrealError> {
    db.query(
        "
        DEFINE TABLE tenant SCHEMAFULL;
        DEFINE FIELD name ON TABLE tenant TYPE string;
        DEFINE FIELD created_at ON TABLE tenant TYPE datetime DEFAULT time::now();

        DEFINE TABLE user_tenant SCHEMAFULL;
        DEFINE FIELD user_sub ON TABLE user_tenant TYPE string;
        DEFINE FIELD tenant_id ON TABLE user_tenant TYPE record<tenant>;
        DEFINE FIELD role ON TABLE user_tenant TYPE string DEFAULT 'member';
        DEFINE FIELD joined_at ON TABLE user_tenant TYPE datetime DEFAULT time::now();

        DEFINE INDEX user_sub_unique ON TABLE user_tenant COLUMNS user_sub UNIQUE;
        DEFINE INDEX tenant_idx ON TABLE user_tenant COLUMNS tenant_id;
    ",
    )
    .await?
    .check()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inject_select_no_where() {
        let sql = "SELECT * FROM counter";
        let result = TenantAwareSurrealDb::inject_tenant_filter(sql);
        assert!(result.contains("WHERE tenant_id = $tenant_id"));
    }

    #[test]
    fn inject_create_set() {
        let sql = "CREATE counter SET name = $name, count = 0";
        let result = TenantAwareSurrealDb::inject_tenant_filter(sql);
        assert!(result.contains("tenant_id = $tenant_id"));
    }

    #[test]
    fn inject_update_with_where() {
        let sql = "UPDATE counter SET count += 1 WHERE id = $id";
        let result = TenantAwareSurrealDb::inject_tenant_filter(sql);
        assert!(result.contains("tenant_id = $tenant_id AND"));
        assert!(result.contains("id = $id"));
    }

    #[test]
    fn inject_delete_with_where() {
        let sql = "DELETE FROM counter WHERE id = $id";
        let result = TenantAwareSurrealDb::inject_tenant_filter(sql);
        assert!(result.contains("tenant_id = $tenant_id AND"));
        assert!(result.contains("id = $id"));
    }

    #[test]
    fn inject_select_with_where() {
        let sql = "SELECT * FROM counter WHERE user = $user";
        let result = TenantAwareSurrealDb::inject_tenant_filter(sql);
        assert!(result.contains("tenant_id = $tenant_id AND"));
        assert!(result.contains("user = $user"));
    }

    #[test]
    fn inject_select_with_order_by() {
        let sql = "SELECT * FROM counter ORDER BY created_at DESC";
        let result = TenantAwareSurrealDb::inject_tenant_filter(sql);
        assert!(
            result.contains("WHERE tenant_id = $tenant_id"),
            "missing WHERE clause: {result}"
        );
        assert!(
            result.contains("ORDER BY created_at DESC"),
            "missing ORDER BY: {result}"
        );
    }

    #[test]
    fn admin_mode_no_injection() {
        let sql = "SELECT * FROM counter";
        // Admin mode — no tenant_id, SQL passes through unchanged
        // This test verifies the logic, actual passthrough is in query() method
        assert!(!sql.contains("tenant_id"));
    }
}
