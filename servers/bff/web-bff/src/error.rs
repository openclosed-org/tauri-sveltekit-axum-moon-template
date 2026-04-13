//! BFF 错误类型 — HTTP 错误映射。

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;

/// BFF 层错误枚举。
#[derive(Debug, thiserror::Error)]
pub enum BffError {
    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Validation failed: {0}")]
    Validation(String),
}

impl IntoResponse for BffError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            BffError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            BffError::Unauthorized(_) => (StatusCode::UNAUTHORIZED, self.to_string()),
            BffError::Validation(_) => (StatusCode::UNPROCESSABLE_ENTITY, self.to_string()),
        };

        (status, Json(json!({ "error": message }))).into_response()
    }
}

/// BFF 结果类型。
pub type BffResult<T> = Result<T, BffError>;
