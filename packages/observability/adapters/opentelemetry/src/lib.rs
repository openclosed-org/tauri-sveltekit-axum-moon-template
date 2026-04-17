//! adapter-telemetry-otel — OpenTelemetry integration for distributed tracing.
//!
//! Provides initialization helpers for OpenTelemetry SDK with OTLP exporter.
//! No-op until initialized by the composition layer (servers/bff or apps/client/native).

use std::env;

use opentelemetry::{KeyValue, global};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{
    Resource,
    propagation::TraceContextPropagator,
    runtime,
    trace::{RandomIdGenerator, Sampler, TracerProvider},
};

/// Initialize OpenTelemetry tracing with OTLP exporter.
/// Returns a tracer provider that should be kept alive.
pub fn init_otel_tracing(service_name: &str) -> Result<Option<TracerProvider>, String> {
    let endpoint = resolve_otlp_endpoint();
    if !otel_enabled() && endpoint.is_none() {
        return Ok(None);
    }

    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(endpoint.unwrap_or_else(|| "http://localhost:4317".to_string()))
        .build()
        .map_err(|error| format!("failed to build OTLP span exporter: {error}"))?;

    let provider = TracerProvider::builder()
        .with_sampler(Sampler::ParentBased(Box::new(Sampler::AlwaysOn)))
        .with_id_generator(RandomIdGenerator::default())
        .with_resource(build_resource(service_name))
        .with_batch_exporter(exporter, runtime::Tokio)
        .build();

    global::set_text_map_propagator(TraceContextPropagator::new());
    global::set_tracer_provider(provider.clone());

    Ok(Some(provider))
}

fn build_resource(service_name: &str) -> Resource {
    let mut attributes = vec![KeyValue::new("service.name", service_name.to_string())];

    if let Some(namespace) = env::var("OTEL_SERVICE_NAMESPACE")
        .ok()
        .or_else(|| env::var("SERVICE_NAMESPACE").ok())
    {
        attributes.push(KeyValue::new("service.namespace", namespace));
    }

    if let Some(environment) = env::var("DEPLOYMENT_ENVIRONMENT")
        .ok()
        .or_else(|| env::var("APP_ENV").ok())
        .or_else(|| env::var("ENVIRONMENT").ok())
    {
        attributes.push(KeyValue::new("deployment.environment", environment));
    }

    Resource::new(attributes)
}

fn resolve_otlp_endpoint() -> Option<String> {
    env::var("OTEL_EXPORTER_OTLP_TRACES_ENDPOINT")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .or_else(|| {
            env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
                .ok()
                .filter(|value| !value.trim().is_empty())
        })
}

fn otel_enabled() -> bool {
    env::var("OTEL_ENABLED")
        .ok()
        .map(|value| parse_bool(&value))
        .unwrap_or(false)
}

fn parse_bool(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "yes" | "on"
    )
}
