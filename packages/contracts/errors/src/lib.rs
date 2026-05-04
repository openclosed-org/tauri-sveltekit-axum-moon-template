//! contracts_errors — Unified error contract definitions.
//!
//! This crate defines the **single source of truth** for error responses
//! across all services and clients. All error types serialize to the same
//! JSON shape, ensuring callers and services share a common understanding.
//!
//! ```json
//! {
//!   "error": {
//!     "code": "validation_error",
//!     "message": "Field 'name' is required",
//!     "details": null
//!   }
//! }
//! ```
//!
//! ## Usage
//!
//! Services define their own internal error enums (via `thiserror`) and
//! convert to [`ApiError`] at the boundary (Axum handler / Tauri command).

#![deny(unused_imports, unused_variables)]

use serde::{Deserialize, Serialize, ser::SerializeStruct};
use utoipa::ToSchema;

// ── Error Response DTO ──────────────────────────────────────────

/// Standardized API error response.
///
/// Every error returned by the API conforms to this shape.
/// Callers depend on this public DTO, not on individual service error enums.
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct ErrorResponse {
    /// Machine-readable error code (e.g., `validation_error`, `not_found`).
    pub code: ErrorCode,
    /// Human-readable error message.
    pub message: String,
    /// Optional additional details for debugging.
    /// Never sent in production responses.
    pub details: Option<serde_json::Value>,
}

impl Serialize for ErrorResponse {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut field_count = 2;
        if self.details.is_some() {
            field_count += 1;
        }

        let mut state = serializer.serialize_struct("ErrorResponse", field_count)?;
        state.serialize_field("code", &self.code)?;
        state.serialize_field("message", &self.message)?;
        if let Some(details) = &self.details {
            state.serialize_field("details", details)?;
        }
        state.end()
    }
}

impl ErrorResponse {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            details: None,
        }
    }

    pub fn with_details(
        code: ErrorCode,
        message: impl Into<String>,
        details: serde_json::Value,
    ) -> Self {
        Self {
            code,
            message: message.into(),
            details: Some(details),
        }
    }
}

// ── Error Codes ─────────────────────────────────────────────────

/// Machine-readable error codes.
///
/// These codes are stable across releases and form the public API contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub enum ErrorCode {
    // ── Client Errors (4xx) ─────────────────────────────────
    /// Request was malformed or invalid.
    BadRequest,
    /// Authentication credentials are missing or invalid.
    Unauthorized,
    /// Authenticated user lacks permission to access this resource.
    Forbidden,
    /// Requested resource does not exist.
    NotFound,
    /// Request conflicts with current state.
    Conflict,
    /// Request entity is semantically invalid (validation failure).
    ValidationError,
    /// Request is too large or too frequent.
    RateLimited,

    // ── Server Errors (5xx) ─────────────────────────────────
    /// Unexpected internal failure.
    InternalError,
    /// Downstream service or dependency unavailable.
    ServiceUnavailable,
    /// Database operation failed.
    DatabaseError,
    /// Cache operation failed.
    CacheError,
    /// External HTTP call failed.
    ExternalServiceError,
}

impl std::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            ErrorCode::BadRequest => "bad_request",
            ErrorCode::Unauthorized => "unauthorized",
            ErrorCode::Forbidden => "forbidden",
            ErrorCode::NotFound => "not_found",
            ErrorCode::Conflict => "conflict",
            ErrorCode::ValidationError => "validation_error",
            ErrorCode::RateLimited => "rate_limited",
            ErrorCode::InternalError => "internal_error",
            ErrorCode::ServiceUnavailable => "service_unavailable",
            ErrorCode::DatabaseError => "database_error",
            ErrorCode::CacheError => "cache_error",
            ErrorCode::ExternalServiceError => "external_service_error",
        };
        write!(f, "{s}")
    }
}

// ── Conversion Helpers ──────────────────────────────────────────

/// Internal result alias used by services.
///
/// Wraps [`ApiError`] for ergonomic `Result<T>` shorthand.
pub type ApiResult<T> = Result<T, ApiError>;

/// Unified internal error type.
///
/// Services define their own granular error enums and implement
/// `From<TheirError> for ApiError` to convert at the boundary.
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Rate limited: {0}")]
    RateLimited(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Cache error: {0}")]
    Cache(String),

    #[error("External service error: {0}")]
    ExternalService(String),
}

impl ApiError {
    /// Convert to the public [`ErrorResponse`] DTO.
    pub fn to_response(&self) -> ErrorResponse {
        let (code, message) = match self {
            ApiError::BadRequest(m) => (ErrorCode::BadRequest, m),
            ApiError::Unauthorized(m) => (ErrorCode::Unauthorized, m),
            ApiError::Forbidden(m) => (ErrorCode::Forbidden, m),
            ApiError::NotFound(m) => (ErrorCode::NotFound, m),
            ApiError::Conflict(m) => (ErrorCode::Conflict, m),
            ApiError::Validation(m) => (ErrorCode::ValidationError, m),
            ApiError::RateLimited(m) => (ErrorCode::RateLimited, m),
            ApiError::Internal(m) => (ErrorCode::InternalError, m),
            ApiError::ServiceUnavailable(m) => (ErrorCode::ServiceUnavailable, m),
            ApiError::Database(m) => (ErrorCode::DatabaseError, m),
            ApiError::Cache(m) => (ErrorCode::CacheError, m),
            ApiError::ExternalService(m) => (ErrorCode::ExternalServiceError, m),
        };
        ErrorResponse::new(code, message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_response_serializes_correctly() {
        let resp = ErrorResponse::new(ErrorCode::NotFound, "User not found");
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("NotFound"));
        assert!(json.contains("User not found"));
    }

    #[test]
    fn error_code_display() {
        assert_eq!(ErrorCode::ValidationError.to_string(), "validation_error");
    }

    #[test]
    fn api_error_to_response_mapping() {
        let err = ApiError::Validation("email is invalid".to_string());
        let resp = err.to_response();
        assert_eq!(resp.code, ErrorCode::ValidationError);
        assert_eq!(resp.message, "email is invalid");
    }
}
