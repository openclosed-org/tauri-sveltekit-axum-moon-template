# Projector Worker

> 目的：说明 `projector` 作为默认 projection/replay 链路样例时，当前提供什么结构，以及哪些能力仍停留在骨架阶段。

## 状态

- status: `reference`
- 角色：默认 projection、checkpoint、replay 结构参考 worker
- 说明：当前已补齐真实 outbox replay、持久化 read model、磁盘 checkpoint，并新增可选 NATS live tail；live tail 默认使用 queue group 降低多副本重复消费，但仍不提供 durable broker checkpoint 语义。同时已补到独立的 dev SOPS/Kustomize/Flux 落点，但默认仍保持 replicas=0，待 shared libSQL/Turso secret 经解密核实后再启用

## 责任

1. 消费 replayable events 并更新 read model。
2. 维护 projection checkpoint 与恢复顺序。
3. 约束 read model 可删可重建，不能成为新的真理源。
4. 为后续 projector/indexer 类能力提供最小结构样例。

## 入口

1. `src/main.rs`：主进程入口、轮询主循环与健康检查。
2. `src/consumers/`：事件消费者定义。
3. `src/readmodels/`：持久化 read model 更新逻辑。
4. `src/source.rs`：从 `counter_outbox` 读取 replayable event source。
5. `src/checkpoint/` 与 `src/replay/`：checkpoint 与 replay 控制。
6. `../../platform/model/deployables/projector-worker.yaml`：deployable 元数据入口。
7. `../../infra/k3s/overlays/dev/projector-worker/kustomization.yaml`：独立 dev overlay 入口。
8. `../../infra/gitops/flux/apps/projector-worker.yaml`：Flux GitOps 入口。

## 验证

```bash
cargo check -p projector-worker
cargo test -p projector-worker
```

## 不要这样用

1. 不要把当前 `outbox replay + optional NATS live tail + queue group` 写成已经完成的最终事件流平台。
2. 不要把 read model 当成业务真理源。
3. 不要跳过 replay/rebuild 约束，只保留“实时消费”这一半能力。
4. 不要把 queue group 误写成 durable consumer；当前可重建能力仍主要来自 outbox replay。
5. 不要在 shared libSQL/Turso secret 仍指向本地 `file:` 路径，或 `just sops-verify-counter-shared-db ENV=dev` 仍未通过时，把 dev overlay 中的 `projector-worker` 直接扩到 1 个副本。
