/// Projector error types.
#[derive(Debug, thiserror::Error)]
pub enum ProjectorError {
    #[error("configuration error: {0}")]
    Configuration(String),

    #[error("source error: {0}")]
    Source(String),

    #[error("Consumer error: {0}")]
    Consumer(String),

    #[error("Read model error: {0}")]
    ReadModel(String),

    #[error("Internal error: {0}")]
    Internal(String),
}
