# Indexer Worker

> 目的：标记 `indexer-worker` 当前仍是索引链路骨架，避免它被误当成已经收敛的默认 worker 模板。

## 状态

- status: `stub`
- 角色：索引构建、源事件拉取与 sink 写入的实验骨架
- 说明：deployable、health endpoint 和主循环已存在，但真实 source 与真实 sink 仍未闭环；当前已补到 envelope-first 的事件入口语义，但还不是默认 reference worker

## 责任

1. 从多个 source 拉取事件并维护 source-level checkpoint。
2. 优先把原始输入解析为 `contracts_events::EventEnvelope`，并在必要时兼容回退到裸 `AppEvent`。
3. 将索引结果写入 sink，并把共享 metadata 继续向下游发布，而不是在下游重猜事件语义。

## 入口

1. `src/main.rs`：主循环与健康检查入口。
2. `src/sources/`：source 抽象与 stub source。
3. `src/transforms/`：原始事件到 canonical `EventEnvelope` 的转换与兼容回退。
4. `src/sinks/` 与 `src/checkpoint/`：sink 写入、共享 metadata 保留与 source checkpoint。
5. `../../platform/model/deployables/indexer-worker.yaml`：deployable 元数据入口。

## 当前已落地的语义

1. `PassthroughTransform` 优先解析 `EventEnvelope`，保留原始 `event id`、`source_service` 与 metadata。
2. 如果输入仍是旧的裸 `AppEvent`，会在 transform 层封装为新的 `EventEnvelope` 并补齐可用 metadata。
3. `IndexedEvent` 现在保存 `EventMetadata`，下游 pubsub 发布时会继续复用这份 metadata。
4. 当前 memory source / memory sink 仍仅用于 stub 模式验证，不代表真实索引基础设施已经接入。

## 验证

```bash
cargo check -p indexer-worker
cargo test -p indexer-worker
```

建议在涉及 shared event semantics 的改动后，再执行：

```bash
just verify-replay strict
just boundary-check
```

## 不要这样用

1. 不要把 memory source 或 memory sink 写成真实索引基础设施已经接好。
2. 不要把它当成默认 reference worker；当前默认异步主链仍优先看 `outbox-relay` 与 `projector`。
3. 不要让索引状态反向变成业务真理源。
4. 不要在 `main.rs` 下游重新手写 event type 或 metadata 推导，shared event semantics 应优先在 transform 层完成。
