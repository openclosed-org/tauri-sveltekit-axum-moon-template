//! OpenFGA adapter — real authorization backend integration.

use async_trait::async_trait;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use crate::ports::{AuthzError, AuthzPort, AuthzTuple, AuthzTupleKey};

#[derive(Debug, Clone)]
pub struct OpenFgaConfig {
    pub endpoint: String,
    pub store_id: String,
    pub authorization_model_id: Option<String>,
}

impl OpenFgaConfig {
    pub fn validate(&self) -> Result<(), AuthzError> {
        if self.endpoint.trim().is_empty() {
            return Err(AuthzError::ConnectionError(
                "openfga endpoint cannot be empty".to_string(),
            ));
        }
        if self.store_id.trim().is_empty() {
            return Err(AuthzError::ConnectionError(
                "openfga store id cannot be empty".to_string(),
            ));
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct OpenFgaAdapter {
    client: reqwest::Client,
    config: OpenFgaConfig,
}

impl OpenFgaAdapter {
    pub fn new(config: OpenFgaConfig) -> Result<Self, AuthzError> {
        config.validate()?;
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .map_err(|error| AuthzError::ConnectionError(error.to_string()))?;
        Ok(Self { client, config })
    }

    fn store_url(&self, path: &str) -> String {
        format!(
            "{}/stores/{}{}",
            self.config.endpoint.trim_end_matches('/'),
            self.config.store_id,
            path
        )
    }

    async fn parse_error(response: reqwest::Response) -> AuthzError {
        let status = response.status();
        let body = response
            .text()
            .await
            .unwrap_or_else(|_| "<failed to read response body>".to_string());

        match status {
            StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => AuthzError::PermissionDenied {
                user: "openfga-client".to_string(),
                relation: "request".to_string(),
                object: body,
            },
            _ => AuthzError::StoreError(format!("openfga {status}: {body}")),
        }
    }

    fn model_id(&self) -> Option<&str> {
        self.config.authorization_model_id.as_deref()
    }
}

#[derive(Debug, Serialize)]
struct CheckRequest<'a> {
    tuple_key: &'a AuthzTupleKey,
    #[serde(skip_serializing_if = "Option::is_none")]
    authorization_model_id: Option<&'a str>,
}

#[derive(Debug, Deserialize)]
struct CheckResponse {
    allowed: bool,
}

#[derive(Debug, Serialize)]
struct TupleWriteEnvelope<'a> {
    tuple_keys: Vec<&'a AuthzTupleKey>,
}

#[derive(Debug, Serialize)]
struct WriteRequest<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    authorization_model_id: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    writes: Option<TupleWriteEnvelope<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    deletes: Option<TupleWriteEnvelope<'a>>,
}

#[derive(Debug, Serialize)]
struct ReadRequest<'a> {
    tuple_key: ReadTupleFilter<'a>,
}

#[derive(Debug, Serialize)]
struct ReadTupleFilter<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    user: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    relation: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    object: Option<&'a str>,
}

#[derive(Debug, Deserialize)]
struct ReadResponse {
    #[serde(default)]
    tuples: Vec<ReadTuple>,
}

#[derive(Debug, Deserialize)]
struct ReadTuple {
    key: AuthzTupleKey,
}

#[async_trait]
impl AuthzPort for OpenFgaAdapter {
    async fn check(&self, user: &str, relation: &str, object: &str) -> Result<bool, AuthzError> {
        let tuple_key = AuthzTupleKey::new(user, relation, object);
        let response = self
            .client
            .post(self.store_url("/check"))
            .json(&CheckRequest {
                tuple_key: &tuple_key,
                authorization_model_id: self.model_id(),
            })
            .send()
            .await
            .map_err(|error| AuthzError::ConnectionError(error.to_string()))?;

        if !response.status().is_success() {
            return Err(Self::parse_error(response).await);
        }

        let body: CheckResponse = response
            .json()
            .await
            .map_err(|error| AuthzError::StoreError(error.to_string()))?;
        Ok(body.allowed)
    }

    async fn write_tuple(&self, tuple: &AuthzTupleKey) -> Result<(), AuthzError> {
        let response = self
            .client
            .post(self.store_url("/write"))
            .json(&WriteRequest {
                authorization_model_id: self.model_id(),
                writes: Some(TupleWriteEnvelope {
                    tuple_keys: vec![tuple],
                }),
                deletes: None,
            })
            .send()
            .await
            .map_err(|error| AuthzError::ConnectionError(error.to_string()))?;

        if !response.status().is_success() {
            return Err(Self::parse_error(response).await);
        }

        Ok(())
    }

    async fn delete_tuple(&self, tuple: &AuthzTupleKey) -> Result<(), AuthzError> {
        let response = self
            .client
            .post(self.store_url("/write"))
            .json(&WriteRequest {
                authorization_model_id: self.model_id(),
                writes: None,
                deletes: Some(TupleWriteEnvelope {
                    tuple_keys: vec![tuple],
                }),
            })
            .send()
            .await
            .map_err(|error| AuthzError::ConnectionError(error.to_string()))?;

        if !response.status().is_success() {
            return Err(Self::parse_error(response).await);
        }

        Ok(())
    }

    async fn list_tuples(
        &self,
        user: Option<&str>,
        relation: Option<&str>,
        object: Option<&str>,
    ) -> Result<Vec<AuthzTuple>, AuthzError> {
        let response = self
            .client
            .post(self.store_url("/read"))
            .json(&ReadRequest {
                tuple_key: ReadTupleFilter {
                    user,
                    relation,
                    object,
                },
            })
            .send()
            .await
            .map_err(|error| AuthzError::ConnectionError(error.to_string()))?;

        if !response.status().is_success() {
            return Err(Self::parse_error(response).await);
        }

        let body: ReadResponse = response
            .json()
            .await
            .map_err(|error| AuthzError::StoreError(error.to_string()))?;

        Ok(body
            .tuples
            .into_iter()
            .map(|tuple| AuthzTuple { key: tuple.key })
            .collect())
    }
}
