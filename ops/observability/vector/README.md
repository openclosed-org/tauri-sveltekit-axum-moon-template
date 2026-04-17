# Vector Log Routing

本目录承载 Phase 3 的日志路由配置。

当前职责：

1. 从 Podman socket 采集容器日志。
2. 为日志补充 `environment` 和 `service` 字段。
3. 把结构化或非结构化日志统一送到 OpenObserve。

注意：

1. 当前默认 source 是 `podman_logs`，不是 `docker_logs`。
2. Rust tracing 的 span 会通过 OTel Collector 走 traces 管道；Vector 继续负责 logs 管道。
3. 排障时优先在 OpenObserve 里用 `service`、`request_id`、`correlation_id`、`trace_id` 关联查询。
