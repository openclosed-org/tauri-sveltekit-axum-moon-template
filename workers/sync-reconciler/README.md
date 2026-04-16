# Sync-Reconciler Worker

> 目的：标记 `sync-reconciler-worker` 当前只是冲突解决与数据对账骨架，而不是已经接入真实同步链路的默认实现。

## 状态

- status: `stub`
- 角色：sync conflict resolution 与 reconciliation 的实验骨架
- 说明：plan、strategy、executor 框架已在，但真实数据对比与真实修复执行尚未落地

## 责任

1. 定义 reconciliation plan 与冲突处理策略。
2. 周期性执行 plan 并记录 reconcile 结果。
3. 为未来 offline-first / multi-source sync 场景保留 worker 入口。

## 入口

1. `src/main.rs`：主循环、health server、plan 组装入口。
2. `src/plans/`：reconciliation plan 与策略定义。
3. `src/executors/`：执行器接口与 stub executor。
4. `src/conflict/` 与 `src/checkpoint/`：冲突处理与状态骨架。
5. `../../platform/model/deployables/sync-reconciler-worker.yaml`：deployable 元数据入口。

## 验证

```bash
cargo check -p sync-reconciler-worker
cargo test -p sync-reconciler-worker
```

## 不要这样用

1. 不要把当前 stub executor 写成真实对账链路已经可用。
2. 不要在没有真实 source comparison 与恢复策略前，把关键同步流程接入这里。
3. 不要把冲突解决逻辑散落到客户端或 server，绕开 worker 的恢复与重试边界。
