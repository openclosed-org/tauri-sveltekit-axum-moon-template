//! Tenant extraction middleware — extracts user_sub from JWT Bearer token.
//!
//! Environment-gated JWT verification:
//! - Dev mode (jwt_secret == "dev-secret-change-in-production"): insecure decode
//!   without signature verification (backward-compatible with Phase 6).
//! - Prod mode: full HS256 signature verification + exp claim validation.
//!
//! Extracts the JWT `sub` claim and injects it as `TenantId` into request extensions.

use axum::{extract::Request, http::StatusCode, middleware::Next, response::Response};
use domain::ports::TenantId;
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode, dangerous::insecure_decode};
use serde::Deserialize;

/// JWT claims we need — only `sub` matters for user identification.
#[derive(Debug, Deserialize)]
struct IdTokenClaims {
    sub: String,
}

/// Default dev secret used by the boilerplate.
const DEV_SECRET: &str = "dev-secret-change-in-production";

/// Extract user_sub from Authorization: Bearer <token> and inject as TenantId.
///
/// Reads `jwt_secret` from the BFF config in request extensions.
pub async fn tenant_middleware(mut req: Request, next: Next) -> Result<Response, StatusCode> {
    // 1. Extract Bearer token from Authorization header
    let token = req
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // 2. Read jwt_secret from the BFF config (injected via Extension layer)
    let jwt_secret: String = req
        .extensions()
        .get::<String>()
        .cloned()
        .unwrap_or_default();

    // 3. Decode JWT — dev mode vs prod mode
    let token_data = if jwt_secret == DEV_SECRET {
        tracing::warn!(
            "BFF JWT: using insecure decode (dev-secret) — set APP_JWT_SECRET for production"
        );
        insecure_decode::<IdTokenClaims>(token).map_err(|e| {
            tracing::debug!(error = %e, "BFF JWT insecure_decode failed");
            StatusCode::UNAUTHORIZED
        })?
    } else {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_exp = true;
        decode::<IdTokenClaims>(
            token,
            &DecodingKey::from_secret(jwt_secret.as_ref()),
            &validation,
        )
        .map_err(|e| {
            tracing::warn!(error = %e, "BFF JWT signature/exp validation failed");
            StatusCode::UNAUTHORIZED
        })?
    };

    // 4. Inject user_sub (JWT sub) as TenantId into request extensions
    req.extensions_mut()
        .insert(TenantId(token_data.claims.sub));

    Ok(next.run(req).await)
}
