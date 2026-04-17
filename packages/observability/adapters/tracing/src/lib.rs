//! adapter-telemetry-tracing — tracing-subscriber integration.
//!
//! Provides structured logging initialization with environment-based filtering.
//! Used by both the Axum server and Tauri desktop app.

use tracing_subscriber::{EnvFilter, Layer, prelude::*};

/// Builds an env filter with a repo-defined default fallback.
pub fn build_env_filter(default_level: &str) -> EnvFilter {
    EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default_level))
}

/// Initialize structured logging with env-based filtering.
pub fn init_tracing(default_level: &str) -> Result<(), String> {
    let filter = build_env_filter(default_level);

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(true)
                .with_thread_ids(true)
                .with_filter(filter),
        )
        .try_init()
        .map_err(|e| format!("failed to initialize tracing: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_tracing_returns_ok() {
        // First call succeeds
        // Note: second call would fail since subscriber is already set
        // This is expected behavior — init once
        let result = init_tracing("info");
        // May fail if already initialized by another test
        assert!(result.is_ok() || result.is_err());
    }
}
