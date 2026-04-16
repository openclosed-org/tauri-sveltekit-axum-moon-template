# Indexing Service

> 目的：标记 `indexing-service` 当前主要是搜索/索引语义占位，而不是已经完成的默认业务 service。

## 状态

- status: `stub`
- 角色：索引与搜索能力的 service 语义保留位
- 说明：当前更多是在平台模型和目录结构上保留位置，真正的执行链路仍以 `workers/indexer/` 为主

## 责任

1. 为 `search-index-snapshot` 这类派生索引实体保留 ownership 位置。
2. 约束索引能力应从 replayable events 或 snapshot 重建。
3. 为未来搜索能力收敛提供稳定 service 名称。

## 入口

1. `model.yaml`：索引语义与查询保留位。
2. `src/`：现有骨架实现。
3. `migrations/`：本地索引状态脚手架。

## 验证

```bash
cargo check -p indexing-service
cargo test -p indexing-service
```

## 不要这样用

1. 不要把它当成新业务 service 的模板。
2. 不要把 service 语义占位误写成已经闭环的搜索产品能力。
3. 不要忽略 `workers/indexer/` 与 replay/rebuild 约束，直接把索引状态当成新的业务真理源。
