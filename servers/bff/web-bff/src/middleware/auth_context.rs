//! Authentication context middleware for the Web BFF.
//!
//! The Axum middleware shell stays in `servers/**`; provider-neutral OIDC token
//! verification is delegated to `authn-oidc-verifier`.

use axum::{extract::Request, http::StatusCode, middleware::Next, response::Response};
use jsonwebtoken::{Algorithm, DecodingKey, Validation, dangerous::insecure_decode, decode};
use serde::Deserialize;

use crate::config::Config;
use crate::error::BffError;
use crate::request_context::{RequestContext, request_id};
use authn_oidc_verifier::{OidcVerifier, VerifiedIdentity};

/// JWT claims used by legacy BFF-local HS256/dev-token fallback.
#[derive(Debug, Deserialize)]
struct IdTokenClaims {
    sub: String,
    tenant_id: Option<String>,
    roles: Option<Vec<String>>,
    email: Option<String>,
    aud: Option<serde_json::Value>,
    iss: Option<String>,
}

const DEV_SECRET: &str = "dev-secret-change-in-production";

/// Extract identity from Authorization/dev headers and inject BFF request context.
pub async fn auth_context_middleware(mut req: Request, next: Next) -> Result<Response, BffError> {
    let config: Config = req
        .extensions()
        .get::<Config>()
        .cloned()
        .unwrap_or_default();

    if config.allows_dev_headers()
        && let Some(context) = RequestContext::from_dev_headers(&req)
    {
        req.extensions_mut().insert(context);
        return Ok(next.run(req).await);
    }

    let token = req
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .map(str::to_owned)
        .ok_or_else(|| BffError::Unauthorized("Missing bearer token".to_string()))?;
    let oidc_verifier = req.extensions().get::<OidcVerifier>().cloned();

    let identity = verify_bearer_token(&token, oidc_verifier.as_ref(), &config)
        .await
        .map_err(|_| BffError::Unauthorized("Invalid bearer token".to_string()))?;

    let context = RequestContext::from_verified_identity(identity, request_id(&req));
    req.extensions_mut().insert(context);

    Ok(next.run(req).await)
}

async fn verify_bearer_token(
    token: &str,
    oidc_verifier: Option<&OidcVerifier>,
    config: &Config,
) -> Result<VerifiedIdentity, StatusCode> {
    if !config.oidc_issuer.trim().is_empty() {
        if let Some(verifier) = oidc_verifier {
            return verifier
                .verify(token)
                .await
                .map_err(|_| StatusCode::UNAUTHORIZED);
        }

        tracing::warn!("BFF OIDC issuer configured without verifier in request extensions");
        return Err(StatusCode::UNAUTHORIZED);
    }

    if config.jwt_secret == DEV_SECRET {
        tracing::warn!(
            "BFF JWT: using insecure decode (dev-secret) — set APP_JWT_SECRET for production"
        );
        return insecure_decode::<IdTokenClaims>(token)
            .map(|token_data| identity_from_claims(token_data.claims))
            .map_err(|error| {
                tracing::debug!(error = %error, "BFF JWT insecure_decode failed");
                StatusCode::UNAUTHORIZED
            });
    }

    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = true;
    validation.validate_aud = false;
    decode::<IdTokenClaims>(
        token,
        &DecodingKey::from_secret(config.jwt_secret.as_ref()),
        &validation,
    )
    .map(|token_data| identity_from_claims(token_data.claims))
    .map_err(|error| {
        tracing::warn!(error = %error, "BFF JWT signature/exp validation failed");
        StatusCode::UNAUTHORIZED
    })
}

fn identity_from_claims(claims: IdTokenClaims) -> VerifiedIdentity {
    VerifiedIdentity {
        sub: claims.sub,
        tenant_id: claims.tenant_id,
        roles: claims.roles.unwrap_or_default(),
        email: claims.email,
        issuer: claims.iss,
        audience: claims.aud,
    }
}
