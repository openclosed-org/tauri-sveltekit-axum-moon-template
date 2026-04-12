use thiserror::Error;

pub type AdminBffResult<T> = Result<T, AdminBffError>;

#[derive(Error, Debug)]
pub enum AdminBffError {
    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),
}

impl axum::response::IntoResponse for AdminBffError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            AdminBffError::Internal(msg) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, msg),
            AdminBffError::NotFound(msg) => (axum::http::StatusCode::NOT_FOUND, msg),
            AdminBffError::Unauthorized(msg) => (axum::http::StatusCode::UNAUTHORIZED, msg),
            AdminBffError::BadRequest(msg) => (axum::http::StatusCode::BAD_REQUEST, msg),
            AdminBffError::ServiceUnavailable(msg) => {
                (axum::http::StatusCode::SERVICE_UNAVAILABLE, msg)
            }
        };

        (status, axum::Json(serde_json::json!({ "error": message }))).into_response()
    }
}
