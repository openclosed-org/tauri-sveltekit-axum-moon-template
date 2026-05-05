# OpenTelemetry Collector

本目录承载本地统一 OTLP 入口配置。

当前约定：

1. Rust 进程统一通过 `OTEL_EXPORTER_OTLP_ENDPOINT` 指向 Collector，而不是直接连 OpenObserve。
2. 本地默认 gRPC 入口是 `http://localhost:4317`。
3. Collector 再把 OTLP traces/metrics/logs 转发到 OpenObserve。
4. 本地 health endpoint 是 `http://localhost:13133/`。

本地启动：

```bash
cargo run -p repo-tools -- infra local observability up
```

应用侧最小环境变量：

```bash
OTEL_ENABLED=true
OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317
```

当前仓库里的 Rust 入口统一走 `observability::init_observability(...)`，会在：

1. `OTEL_ENABLED=true` 时启用 OTLP trace 导出。
2. `OTEL_EXPORTER_OTLP_ENDPOINT` 已设置时自动启用 OTLP trace 导出。
3. 其余情况下退回本地结构化日志，不阻塞开发环境。
