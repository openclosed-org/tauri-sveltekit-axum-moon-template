//! API route handlers and request/response adapters.
//!
//! Note: Generic handlers are not directly usable with axum's routing macros.
//! The BFF/server layer should construct the UserService and wire it to handlers.

use crate::application::{InitTenantInput, UserService};
use crate::contracts::{InitTenantRequest, InitTenantResponse};
use crate::infrastructure::{LibSqlTenantRepository, LibSqlUserRepository, LibSqlUserTenantRepository};
use ::domain::ports::lib_sql::LibSqlPort;

/// Type alias for the concrete user service used in the web-bff.
pub type WebUserService<P> = UserService<
    LibSqlUserRepository<P>,
    LibSqlTenantRepository<P>,
    LibSqlUserTenantRepository<P>,
>;

/// Build a user service instance from a LibSQL port.
pub fn build_user_service<P: LibSqlPort + Clone>(db: P) -> WebUserService<P> {
    UserService::new(
        LibSqlUserRepository::new(db.clone()),
        LibSqlTenantRepository::new(db.clone()),
        LibSqlUserTenantRepository::new(db),
    )
}
