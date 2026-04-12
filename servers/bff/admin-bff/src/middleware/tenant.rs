use axum::{
    extract::Request,
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use kernel::TenantId;

/// Extract tenant ID from JWT in Authorization header for admin BFF
pub async fn admin_tenant_middleware(
    headers: HeaderMap,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = headers
        .get(axum::http::header::AUTHORIZATION)
        .ok_or(StatusCode::UNAUTHORIZED)?
        .to_str()
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    if !auth_header.starts_with("Bearer ") {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let token = &auth_header["Bearer ".len()..];

    // Dev mode: insecure decode (same as api and web-bff)
    let tenant_id = decode_tenant_id_insecure(token)?;

    req.extensions_mut().insert(tenant_id);

    Ok(next.run(req).await)
}

/// Decode tenant ID from JWT — dev mode only
fn decode_tenant_id_insecure(token: &str) -> Result<TenantId, StatusCode> {
    use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};

    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let payload_bytes = URL_SAFE_NO_PAD
        .decode(parts[1])
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    let payload: serde_json::Value =
        serde_json::from_slice(&payload_bytes).map_err(|_| StatusCode::UNAUTHORIZED)?;

    let sub = payload
        .get("sub")
        .and_then(|v| v.as_str())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    Ok(TenantId(sub.to_string()))
}
