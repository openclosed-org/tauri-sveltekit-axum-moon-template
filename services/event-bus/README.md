# Event Bus — 事件总线

> 服务间异步通信抽象。EventBus trait + Outbox Pattern。
> 当前实现：内存广播（tokio broadcast channels）。

```bash
cargo test -p event-bus
cargo build -p event-bus
```

架构说明见 [services/README.md](../README.md)。
