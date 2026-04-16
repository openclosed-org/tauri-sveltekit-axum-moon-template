# Local Development Infrastructure

> 目的：说明 `infra/local/` 当前在后端默认开发轨道中负责什么、从哪里进入，以及哪些旧的本地开发说法不应继续传播。

## 状态

- status: `implemented`
- 角色：本地基础依赖承载层
- 说明：这里负责把 NATS、Valkey、MinIO 和可选 sqld 等基础设施拉起，但它不是 `.env` 教程目录

## 责任

1. 提供本地基础依赖的启动、停止、状态与日志入口。
2. 复用 `infra/docker/compose/core.yaml` 管理开发期基础设施。
3. 为 `counter-service` 默认参考链提供最小本地承载环境。

## 入口

1. `scripts/bootstrap.sh`：默认本地基础设施入口。
2. `../docker/compose/core.yaml`：核心基础依赖定义。
3. `../../docs/operations/local-dev.md`：本地开发主说明。
4. `../../justfiles/sops.just`：本地 secrets 注入默认入口。

## 常用命令

```bash
bash infra/local/scripts/bootstrap.sh up
bash infra/local/scripts/bootstrap.sh status
bash infra/local/scripts/bootstrap.sh logs
bash infra/local/scripts/bootstrap.sh down
```

当前脚本会管理的核心依赖包括：

1. NATS
2. Valkey
3. MinIO
4. 可选的 sqld/libSQL client-server 形态

## 验证

```bash
just validate-deps
bash infra/local/scripts/bootstrap.sh status
```

## 不要这样用

1. 不要把 `.env` 写成当前默认本地 secrets 路径；后端默认应优先看 `just sops-run`。
2. 不要把前端或桌面壳层启动步骤写成所有后端任务的默认前置条件。
3. 不要在这个 README 里重复维护所有 compose 细节，真实配置以脚本和 compose 文件为准。
