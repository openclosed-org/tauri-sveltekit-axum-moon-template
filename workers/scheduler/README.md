# Scheduler Worker

> 目的：标记 `scheduler-worker` 当前只是时间驱动任务分发骨架，而不是已经收敛的默认后台调度实现。

## 状态

- status: `stub`
- 角色：时间驱动任务注册与分发骨架
- 说明：job registry 与 dispatcher 结构已在，但真实 cron 求值和真实任务执行尚未完成

## 责任

1. 维护定时任务注册表与启用状态。
2. 周期性评估 schedule 并把到期任务交给 executor。
3. 为后续后台调度能力预留稳定 worker 入口。

## 入口

1. `src/main.rs`：主循环、健康检查与 registry 组装。
2. `src/jobs/`：任务注册模型。
3. `src/dispatch/`：任务分发与 executor 抽象。
4. `src/dedupe/`：任务级去重骨架。
5. `../../platform/model/deployables/scheduler-worker.yaml`：deployable 元数据入口。

## 验证

```bash
cargo check -p scheduler-worker
cargo test -p scheduler-worker
```

## 不要这样用

1. 不要把当前“只记录日志”的 stub 行为写成真实调度系统已可用。
2. 不要在没有声明幂等、重试、恢复顺序前，把业务关键任务接到这个骨架上。
3. 不要把时间触发逻辑直接散落到 server 或 service 里，绕开 worker 边界。
