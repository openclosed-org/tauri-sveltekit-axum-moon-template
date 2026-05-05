//! BFF-local HTTP extractors and header helpers.

use axum::{
    Json,
    extract::{FromRequest, Request, rejection::JsonRejection},
    http::HeaderMap,
};

use crate::error::{BffError, BffResult};

pub struct ContractJson<T>(pub T);

impl<S, T> FromRequest<S> for ContractJson<T>
where
    Json<T>: FromRequest<S, Rejection = JsonRejection>,
    S: Send + Sync,
{
    type Rejection = BffError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        match Json::<T>::from_request(req, state).await {
            Ok(Json(value)) => Ok(Self(value)),
            Err(JsonRejection::MissingJsonContentType(_)) => Err(BffError::UnsupportedMediaType(
                "Expected content-type: application/json".to_string(),
            )),
            Err(JsonRejection::JsonSyntaxError(_)) => Err(BffError::BadRequest(
                "Malformed JSON request body".to_string(),
            )),
            Err(JsonRejection::JsonDataError(error)) => {
                Err(BffError::Validation(error.body_text()))
            }
            Err(JsonRejection::BytesRejection(error))
                if error.body_text().contains("length limit") =>
            {
                Err(BffError::PayloadTooLarge(
                    "Request body too large".to_string(),
                ))
            }
            Err(error) => Err(BffError::BadRequest(error.body_text())),
        }
    }
}

pub fn idempotency_key(headers: &HeaderMap) -> BffResult<Option<String>> {
    let Some(value) = headers.get("Idempotency-Key") else {
        return Ok(None);
    };

    let key = value
        .to_str()
        .map_err(|_| BffError::BadRequest("Idempotency-Key must be valid ASCII".to_string()))?;
    let key = key.trim();
    if key.is_empty() {
        return Err(BffError::BadRequest(
            "Idempotency-Key must not be empty".to_string(),
        ));
    }

    Ok(Some(key.to_string()))
}
