//! Port types and newtypes for cross-crate use.

pub mod lib_sql;
pub mod surreal_db;

/// Tenant identifier — extracted from JWT `sub` claim.
///
/// Injected into Axum request extensions by tenant middleware.
/// Used by TenantAwareSurrealDb to scope all queries.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TenantId(pub String);

impl TenantId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for TenantId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
