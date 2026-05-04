//! SDK Counter HTTP — HTTP client backend for web and desktop apps.
//!
//! This crate implements `sdk_counter::CounterService` over HTTP,
//! calling the BFF's `/api/counter/*` endpoints.
//!
//! ## Usage
//! ```ignore
//! use sdk_counter_http::CounterHttpClient;
//! use sdk_counter::CounterService;
//!
//! let client = CounterHttpClient::new("http://localhost:3000", "jwt-token");
//! let value = client.increment(&CounterId::new("tenant-1"), None).await?;
//! ```
use async_trait::async_trait;
use reqwest::StatusCode;
use sdk_counter::{CounterError, CounterId, CounterService};

/// HTTP response body from BFF counter endpoints.
#[derive(Debug, serde::Deserialize)]
struct CounterResponse {
    value: i64,
}

/// HTTP client for counter operations via BFF.
///
/// Sends requests to BFF endpoints with Bearer token authentication.
pub struct CounterHttpClient {
    client: reqwest::Client,
    base_url: String,
    auth_token: String,
}

impl CounterHttpClient {
    /// Create a new HTTP counter client.
    ///
    /// - `base_url`: BFF base URL (e.g. `http://localhost:3000`)
    /// - `auth_token`: JWT Bearer token for authentication
    pub fn new(base_url: &str, auth_token: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
            auth_token: auth_token.to_string(),
        }
    }

    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    fn auth_header_value(&self) -> String {
        format!("Bearer {}", self.auth_token)
    }

    async fn parse_counter_response(response: reqwest::Response) -> Result<i64, CounterError> {
        let status = response.status();
        if status == StatusCode::NOT_FOUND {
            let body: serde_json::Value = response.json().await.unwrap_or(serde_json::Value::Null);
            let msg = body
                .get("error")
                .and_then(|v| v.as_str())
                .unwrap_or("Counter not found")
                .to_string();
            return Err(CounterError::NotFound(msg));
        }

        if !status.is_success() {
            let body: serde_json::Value = response.json().await.unwrap_or(serde_json::Value::Null);
            let msg = body
                .get("error")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown error")
                .to_string();
            return Err(CounterError::Database(format!(
                "HTTP {}: {}",
                status.as_u16(),
                msg
            )));
        }

        let counter: CounterResponse = response
            .json()
            .await
            .map_err(|e| CounterError::Database(format!("Failed to parse response: {}", e)))?;

        Ok(counter.value)
    }
}

#[async_trait]
impl CounterService for CounterHttpClient {
    async fn get_value(&self, counter_id: &CounterId) -> Result<i64, CounterError> {
        let response = self
            .client
            .get(self.url("/api/counter/value"))
            .header("Authorization", self.auth_header_value())
            .header("X-Tenant-Id", counter_id.as_str())
            .send()
            .await
            .map_err(|e| CounterError::Database(format!("HTTP request failed: {}", e)))?;

        Self::parse_counter_response(response).await
    }

    async fn increment(
        &self,
        counter_id: &CounterId,
        idempotency_key: Option<&str>,
    ) -> Result<i64, CounterError> {
        let mut request = self
            .client
            .post(self.url("/api/counter/increment"))
            .header("Authorization", self.auth_header_value())
            .header("X-Tenant-Id", counter_id.as_str());
        if let Some(key) = idempotency_key {
            request = request.header("Idempotency-Key", key);
        }
        let response = request
            .send()
            .await
            .map_err(|e| CounterError::Database(format!("HTTP request failed: {}", e)))?;

        Self::parse_counter_response(response).await
    }

    async fn decrement(
        &self,
        counter_id: &CounterId,
        idempotency_key: Option<&str>,
    ) -> Result<i64, CounterError> {
        let mut request = self
            .client
            .post(self.url("/api/counter/decrement"))
            .header("Authorization", self.auth_header_value())
            .header("X-Tenant-Id", counter_id.as_str());
        if let Some(key) = idempotency_key {
            request = request.header("Idempotency-Key", key);
        }
        let response = request
            .send()
            .await
            .map_err(|e| CounterError::Database(format!("HTTP request failed: {}", e)))?;

        Self::parse_counter_response(response).await
    }

    async fn reset(
        &self,
        counter_id: &CounterId,
        idempotency_key: Option<&str>,
    ) -> Result<i64, CounterError> {
        let mut request = self
            .client
            .post(self.url("/api/counter/reset"))
            .header("Authorization", self.auth_header_value())
            .header("X-Tenant-Id", counter_id.as_str());
        if let Some(key) = idempotency_key {
            request = request.header("Idempotency-Key", key);
        }
        let response = request
            .send()
            .await
            .map_err(|e| CounterError::Database(format!("HTTP request failed: {}", e)))?;

        Self::parse_counter_response(response).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn increment_sends_idempotency_key_header() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/api/counter/increment")
            .match_header("authorization", "Bearer token")
            .match_header("x-tenant-id", "tenant-a")
            .match_header("idempotency-key", "idem-123")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"value":1}"#)
            .create_async()
            .await;

        let client = CounterHttpClient::new(&server.url(), "token");
        let value = client
            .increment(&CounterId::new("tenant-a"), Some("idem-123"))
            .await
            .unwrap();

        assert_eq!(value, 1);
        mock.assert_async().await;
    }
}
