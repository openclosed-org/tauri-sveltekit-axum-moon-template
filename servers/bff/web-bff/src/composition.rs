//! Composition aliases owned by the Web BFF bootstrap.

use counter_service::{
    application::RepositoryBackedCounterService, infrastructure::LibSqlCounterRepository,
};
use std::sync::Arc;
use storage_turso::TursoBackend;
use tenant_service::{
    application::TenantService,
    infrastructure::libsql_adapter::LibSqlTenantRepository as TenantServiceRepository,
};
use user_service::infrastructure::{
    LibSqlTenantRepository as UserTenantInfoRepository, LibSqlUserRepository,
    LibSqlUserTenantRepository,
};

pub type CounterServiceHandle =
    Arc<RepositoryBackedCounterService<LibSqlCounterRepository<TursoBackend>>>;
pub type TenantServiceHandle = Arc<TenantService<TenantServiceRepository<TursoBackend>>>;
pub type UserProfileRepositoryHandle = Arc<LibSqlUserRepository<TursoBackend>>;
pub type UserTenantRepositoryHandle = Arc<LibSqlUserTenantRepository<TursoBackend>>;
pub type UserTenantInfoRepositoryHandle = Arc<UserTenantInfoRepository<TursoBackend>>;
