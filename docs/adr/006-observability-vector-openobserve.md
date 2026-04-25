# ADR-006: Observability with Vector + OpenObserve

## Status
- [x] Proposed
- [ ] Accepted
- [ ] Deprecated
- [ ] Superseded

> **Implementation Status**: Observability is still partial.
> Vector configuration, OpenObserve deployment, and dashboards are all deferred.
> Do not assume observability infrastructure is available.

## Context
The system needs comprehensive observability across:
- **Logging**: Structured, searchable, retained appropriately
- **Metrics**: Business and infrastructure metrics, alerting
- **Tracing**: Distributed tracing across services and workers
- **Baggage**: Cross-cutting correlation IDs

Traditional observability stacks (ELK, Grafana, Jaeger) require multiple tools with different configurations, increasing operational overhead. Options considered:
1. **ELK Stack**: Powerful but complex and resource-heavy
2. **Grafana + Prometheus + Loki + Tempo**: Comprehensive but fragmented
3. **Datadog**: Easy but expensive
4. **Vector + OpenObserve**: Unified, modern, self-hosted
5. **Honeycomb**: Powerful but cloud-only

## Decision
We keep **Vector + OpenObserve** as the preferred observability direction, but not as a fully landed current-state platform.

### Vector (Data Pipeline)
- High performance: Written in Rust, low resource usage
- Versatile: Collects logs, metrics, traces from multiple sources
- Transformable: Filter, enrich, transform data in transit
- Reliable: Disk buffering, retry logic
- Open source: Apache 2.0 license
- Cloud-native: Kubernetes integration

### OpenObserve (Observability Platform)
- Unified: Logs, metrics, traces in single platform
- Compatible: Prometheus, Jaeger, OTLP protocols
- Performant: 10x better storage efficiency than Elasticsearch
- Easy: Single binary deployment
- Cost-effective: Low resource requirements
- Self-hosted: Full control over data

### Intended Structure
```
packages/observability/
├── tracing/      # OpenTelemetry tracing setup
├── metrics/      # Metrics collection and export
├── logging/      # Structured logging configuration
├── baggage/      # Distributed context propagation
└── otel/         # OpenTelemetry SDK integration

infra/local/compose/observability.yaml  # Local observability stack
infra/k3s/...                           # future cluster deployment path
```

### Data Flow
```
Services/Servers/Workers
  └── OpenTelemetry SDK (traces, metrics, logs)
        └── Vector (collect, transform, forward)
              └── OpenObserve (store, visualize, alert)
```

### Telemetry Standards
- **Tracing**: W3C Trace Context, OpenTelemetry SDK
- **Metrics**: Prometheus format, business + infra metrics
- **Logging**: JSON structured logs, correlation IDs
- **Baggage**: Cross-service context propagation

### Rationale
1. **Unified platform**: One tool instead of three (logs/metrics/traces)
2. **Performance**: Rust-based Vector is fast and lightweight
3. **Cost**: OpenObserve is more efficient than ELK
4. **Simplicity**: Fewer components to maintain
5. **Open standards**: OTLP, Prometheus, OpenTelemetry compatibility

## Consequences
### What becomes easier
- Keeping a single future observability direction documented
- Building toward correlated traces, logs, and metrics later

### What becomes more difficult
- New concepts: OpenObserve is newer than ELK/Grafana
- Community: Smaller community than established tools
- Migration: Moving from other stacks requires data migration
- Dashboards: Need to build custom dashboards

### Trade-offs
- **Pros**: Unified, performant, cost-effective, open standards
- **Cons**: Newer tool, smaller community, custom dashboards needed

### Implementation Status
- ✅ Observability package structure defined
- ✅ OpenTelemetry-related package scaffolding exists
- ⏳ Vector configuration for local dev
- ⏳ OpenObserve deployment manifests
- ⏳ Dashboard and alerting setup

## References
- `packages/observability/` - Observability package
- `platform/model/resources/observability.yaml` - Resource definition
- [Vector Documentation](https://vector.dev/docs/)
- [OpenObserve Documentation](https://openobserve.ai/docs/)
- [OpenTelemetry](https://opentelemetry.io/docs/)
