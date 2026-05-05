//! BFF 错误类型 — HTTP 错误映射。

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use contracts_errors::{ErrorCode, ErrorResponse};

/// BFF 层错误枚举。
#[derive(Debug, thiserror::Error)]
pub enum BffError {
    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Dependency failure: {0}")]
    Dependency(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Unsupported media type: {0}")]
    UnsupportedMediaType(String),

    #[error("Validation failed: {0}")]
    Validation(String),

    #[error("Payload too large: {0}")]
    PayloadTooLarge(String),
}

impl IntoResponse for BffError {
    fn into_response(self) -> Response {
        let (status, response) = match self {
            BffError::BadRequest(message) => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new(ErrorCode::BadRequest, message),
            ),
            BffError::Internal(message) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorResponse::new(ErrorCode::InternalError, message),
            ),
            BffError::Dependency(message) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorResponse::new(ErrorCode::DatabaseError, message),
            ),
            BffError::Unauthorized(message) => (
                StatusCode::UNAUTHORIZED,
                ErrorResponse::new(ErrorCode::Unauthorized, message),
            ),
            BffError::Forbidden(message) => (
                StatusCode::FORBIDDEN,
                ErrorResponse::new(ErrorCode::Forbidden, message),
            ),
            BffError::NotFound(message) => (
                StatusCode::NOT_FOUND,
                ErrorResponse::new(ErrorCode::NotFound, message),
            ),
            BffError::Conflict(message) => (
                StatusCode::CONFLICT,
                ErrorResponse::new(ErrorCode::Conflict, message),
            ),
            BffError::UnsupportedMediaType(message) => (
                StatusCode::UNSUPPORTED_MEDIA_TYPE,
                ErrorResponse::new(ErrorCode::BadRequest, message),
            ),
            BffError::Validation(message) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                ErrorResponse::new(ErrorCode::ValidationError, message),
            ),
            BffError::PayloadTooLarge(message) => (
                StatusCode::PAYLOAD_TOO_LARGE,
                ErrorResponse::new(ErrorCode::RateLimited, message),
            ),
        };

        (status, Json(response)).into_response()
    }
}

/// BFF 结果类型。
pub type BffResult<T> = Result<T, BffError>;
