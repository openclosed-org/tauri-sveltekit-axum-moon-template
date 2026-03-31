//! Tenant extraction middleware — extracts tenant_id from JWT Bearer token.
//!
//! Decodes the id_token payload (without signature verification — v1)
//! and injects TenantId into request extensions for downstream handlers.

use axum::{extract::Request, http::StatusCode, middleware::Next, response::Response};
use domain::ports::TenantId;
use jsonwebtoken::dangerous::insecure_decode;
use serde::Deserialize;

/// JWT claims we need — only `sub` matters for tenant identification.
#[derive(Debug, Deserialize)]
struct IdTokenClaims {
    sub: String,
}

/// Extract tenant_id from Authorization: Bearer <id_token> header.
///
/// Uses `dangerous::insecure_decode` — payload-only decode without signature
/// verification. This is acceptable for v1 (consistent with Phase 6 decision).
/// v2 will add JWKS-based full verification.
///
/// On failure: returns 401 UNAUTHORIZED.
pub async fn tenant_middleware(mut req: Request, next: Next) -> Result<Response, StatusCode> {
    // 1. Extract Bearer token from Authorization header
    let token = req
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // 2. Decode JWT payload (no signature verification — v1)
    let token_data =
        insecure_decode::<IdTokenClaims>(token).map_err(|_| StatusCode::UNAUTHORIZED)?;

    // 3. Inject tenant_id into request extensions
    req.extensions_mut().insert(TenantId(token_data.claims.sub));

    Ok(next.run(req).await)
}

#[cfg(test)]
mod tests {
    use super::*;
    use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};

    fn make_test_token(sub: &str) -> String {
        #[derive(serde::Serialize)]
        struct Claims {
            sub: String,
            exp: usize,
        }
        encode(
            &Header::new(Algorithm::HS256),
            &Claims {
                sub: sub.to_string(),
                exp: 9999999999,
            },
            &EncodingKey::from_secret(b"test-secret"),
        )
        .unwrap()
    }

    #[test]
    fn extract_sub_from_valid_jwt() {
        let token = make_test_token("google-sub-123");
        let claims: IdTokenClaims = insecure_decode::<IdTokenClaims>(&token).unwrap().claims;
        assert_eq!(claims.sub, "google-sub-123");
    }

    #[test]
    fn reject_invalid_jwt_format() {
        let result = insecure_decode::<IdTokenClaims>("not-a-jwt");
        assert!(result.is_err());
    }

    #[test]
    fn reject_empty_token() {
        let result = insecure_decode::<IdTokenClaims>("");
        assert!(result.is_err());
    }
}
