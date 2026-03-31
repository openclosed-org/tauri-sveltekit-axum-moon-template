//! Application error types using thiserror and anyhow.
//!
//! 2026 Rust Best Practices:
//! - thiserror for library errors (defines error types)
//! - anyhow for application errors (context-aware error handling)
//! - tracing-error for enriching errors with tracing diagnostics

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::{Value, json};
use std::error::Error;
use thiserror::Error;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] Box<dyn Error + Send + Sync>),

    #[error("Database error: {0}")]
    DatabaseDirect(#[from] surrealdb::Error),

    #[error("Cache error: {0}")]
    Cache(String),

    #[error("HTTP client error: {0}")]
    HttpClient(#[from] reqwest::Error),

    #[error("Authentication error: {0}")]
    Auth(String),

    #[error("Authorization error: {0}")]
    Forbidden(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Internal server error: {0}")]
    Internal(String),

    #[error("Configuration error: {0}")]
    Config(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_code, message) = match &self {
            AppError::Database(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "database_error",
                e.to_string(),
            ),
            AppError::DatabaseDirect(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "database_error",
                e.to_string(),
            ),
            AppError::Cache(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "cache_error",
                e.to_string(),
            ),
            AppError::HttpClient(e) => {
                (StatusCode::BAD_GATEWAY, "http_client_error", e.to_string())
            }
            AppError::Auth(e) => (StatusCode::UNAUTHORIZED, "auth_error", e.to_string()),
            AppError::Forbidden(e) => (StatusCode::FORBIDDEN, "forbidden", e.to_string()),
            AppError::Validation(e) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "validation_error",
                e.to_string(),
            ),
            AppError::NotFound(e) => (StatusCode::NOT_FOUND, "not_found", e.to_string()),
            AppError::Internal(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error",
                e.to_string(),
            ),
            AppError::Config(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "config_error",
                e.to_string(),
            ),
        };

        let body: Value = json!({
            "error": {
                "code": error_code,
                "message": message,
            }
        });

        (status, Json(body)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = AppError::NotFound("User not found".to_string());
        assert_eq!(err.to_string(), "Not found: User not found");
    }

    #[test]
    fn test_error_into_response() {
        let err = AppError::NotFound("User not found".to_string());
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }
}
