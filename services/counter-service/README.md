# Counter Service

> 目的：说明 `counter-service` 作为默认后端参考锚点时，当前负责什么、从哪里进入、以及哪些地方不能过度解读。

## 状态

- status: `reference`
- 角色：默认后端参考锚点，承载最小业务主链与工程横切链的起点
- 说明：它是参考样例，不是玩具 demo；但独立 deployable 与完整 projector 闭环仍未全部完成

## 责任

1. 维护 `counter` 聚合的 service-local semantics。
2. 提供单聚合、CAS、idempotency、outbox write 的最小参考实现。
3. 作为新增后端 service 的首选 copy target。

## 入口

1. `model.yaml`：service-local semantics 真理源。
2. `src/domain/`：聚合与领域规则。
3. `src/application/service.rs`：命令编排、CAS、idempotency、outbox 写入。
4. `src/infrastructure/libsql_adapter.rs`：当前 libSQL 适配实现。
5. `migrations/001_create_counter.sql`：主状态与 outbox 相关表结构。
6. `workers/projector/`：当前 counter 链路的 projection/replay 参考落点。

## 验证

```bash
cargo check -p counter-service
cargo test -p counter-service
```

## 不要这样用

1. 不要把它当成“最小 demo”而忽略工程链路要求。
2. 不要绕过 `model.yaml` 和 shared contracts 直接把实现当成真理源。
3. 不要因为它当前主要由 `web-bff` 以内嵌库方式使用，就推断独立 deployable 已经闭环。
4. 不要忽略它与 `outbox-relay` / `projector` 的 projection/replay 关系，只把它当成同步写模型样例。
