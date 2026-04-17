//! Observability layer — aggregation crate for telemetry adapters.
//!
//! Re-exports from adapter subcrates:
//! - `opentelemetry` — OpenTelemetry integration
//! - `tracing` — tracing-subscriber integration

use ::opentelemetry::trace::{SpanContext, TraceContextExt, TracerProvider as _};
use ::tracing::Span;
use opentelemetry_sdk::trace::TracerProvider as SdkTracerProvider;
use tracing_opentelemetry::OpenTelemetrySpanExt;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// Re-export adapter crates
pub use adapter_telemetry_otel as opentelemetry;
pub use adapter_telemetry_tracing as tracing;

/// Keeps the OpenTelemetry tracer provider alive for the process lifetime.
pub struct ObservabilityGuard {
    tracer_provider: Option<SdkTracerProvider>,
}

impl Drop for ObservabilityGuard {
    fn drop(&mut self) {
        if let Some(provider) = self.tracer_provider.take()
            && let Err(error) = provider.shutdown()
        {
            eprintln!("failed to shut down tracer provider: {error}");
        }
    }
}

/// Current span identifiers used to bridge request traces into event metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceContextIds {
    pub trace_id: String,
    pub span_id: String,
}

/// Initializes structured logging and, when enabled, OTLP trace export.
pub fn init_observability(
    service_name: &str,
    default_level: &str,
) -> Result<ObservabilityGuard, String> {
    let filter = tracing::build_env_filter(default_level);
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_thread_ids(true);

    if let Some(provider) = opentelemetry::init_otel_tracing(service_name)? {
        let tracer = provider.tracer(service_name.to_string());
        let otel_layer = tracing_opentelemetry::layer()
            .with_tracer(tracer)
            .with_location(true)
            .with_level(true)
            .with_threads(true);

        tracing_subscriber::registry()
            .with(filter)
            .with(fmt_layer)
            .with(otel_layer)
            .try_init()
            .map_err(|error| format!("failed to initialize observability subscriber: {error}"))?;

        return Ok(ObservabilityGuard {
            tracer_provider: Some(provider),
        });
    }

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt_layer)
        .try_init()
        .map_err(|error| format!("failed to initialize tracing subscriber: {error}"))?;

    Ok(ObservabilityGuard {
        tracer_provider: None,
    })
}

/// Returns the current span's trace identifiers when OpenTelemetry is active.
pub fn current_trace_context() -> Option<TraceContextIds> {
    let context = Span::current().context();
    let span_context: SpanContext = context.span().span_context().clone();

    span_context.is_valid().then(|| TraceContextIds {
        trace_id: span_context.trace_id().to_string(),
        span_id: span_context.span_id().to_string(),
    })
}
