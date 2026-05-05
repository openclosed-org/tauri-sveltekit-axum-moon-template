//! BFF tenant context resolution for authenticated requests.

use user_service::ports::UserTenantRepository;

use crate::error::{BffError, BffResult};
use crate::request_context::RequestContext;
use crate::state::BffState;

pub async fn resolve_tenant_id(
    state: &BffState,
    request_context: &RequestContext,
) -> BffResult<kernel::TenantId> {
    let user_sub = &request_context.user_sub;
    let binding_repo = state
        .user_tenant_repository()
        .ok_or_else(|| BffError::Internal("Database not initialized".to_string()))?;
    let tenant_id = binding_repo
        .find_user_tenant(user_sub)
        .await
        .map_err(map_tenant_resolution_error)?
        .map(|binding| binding.tenant_id);

    let resolved = tenant_id.map(kernel::TenantId).ok_or_else(|| {
        BffError::Unauthorized("No tenant binding found for authenticated user".to_string())
    })?;

    if let Some(claim_tenant_id) = request_context.tenant_id.as_deref()
        && claim_tenant_id != resolved.as_str()
    {
        tracing::warn!(
            user_sub = %request_context.user_sub,
            claim_tenant_id,
            resolved_tenant_id = %resolved,
            "tenant claim does not match persisted tenant binding"
        );
        return Err(BffError::Forbidden(
            "Tenant claim does not match authenticated user binding".to_string(),
        ));
    }

    Ok(resolved)
}

fn map_tenant_resolution_error(error: user_service::domain::error::UserError) -> BffError {
    let message = error.to_string();
    if message.contains("no such table: user_tenant") {
        return BffError::Unauthorized(
            "No tenant binding found for authenticated user".to_string(),
        );
    }

    BffError::Dependency(format!("Failed to resolve tenant binding: {message}"))
}
