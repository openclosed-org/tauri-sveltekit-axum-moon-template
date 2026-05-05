# Vector Log Routing

本目录承载本地可选日志路由配置。它不是默认 observability 启动路径的一部分。

当前职责：

1. 通过 Podman 的 Docker-compatible socket 采集容器日志。
2. 为日志补充 `environment` 和 `service` 字段。
3. 把结构化或非结构化日志统一送到 OpenObserve。

注意：

1. 当前 source type 是 `docker_logs`，通过 `DOCKER_HOST=unix:///var/run/docker.sock` 连接 Podman socket。Vector `0.55` 没有 Podman-native log source。
2. Vector 在 `infra/docker/compose/observability.yaml` 中属于 `logs` profile；默认 observability smoke 只启动 OpenObserve 与 OTel Collector。
3. Rust tracing 的 span 会通过 OTel Collector 走 traces 管道；Vector 只负责可选容器 logs 管道。
4. 排障时优先在 OpenObserve 里用 `service`、`request_id`、`correlation_id`、`trace_id` 关联查询。
