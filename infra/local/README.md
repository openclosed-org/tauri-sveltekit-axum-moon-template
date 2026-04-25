# Local Development Infrastructure

`infra/local/` is the local dependency layer for the default backend path. It is not a `.env` tutorial directory.

## 入口

1. `scripts/bootstrap.sh`：默认本地基础设施入口。
2. `scripts/bootstrap-auth.sh`：本地 auth 栈入口。
3. `../../docs/operations/local-dev.md`：本地开发主说明。
4. `../../justfiles/sops.just`：本地 secrets 注入默认入口。

## 常用命令

```bash
bash infra/local/scripts/bootstrap.sh up
bash infra/local/scripts/bootstrap.sh status
bash infra/local/scripts/bootstrap.sh logs
bash infra/local/scripts/bootstrap.sh down

bash infra/local/scripts/bootstrap-auth.sh bootstrap
bash infra/local/scripts/bootstrap-auth.sh status
bash infra/local/scripts/bootstrap-auth.sh logs
bash infra/local/scripts/bootstrap-auth.sh down
```

当前脚本会管理的核心依赖包括：

1. NATS
2. Valkey
3. MinIO
4. 可选的 sqld/libSQL client-server 形态
5. 可选的本地 auth 栈：`Zitadel + OpenFGA`

## 不要这样用

1. 不要把 `.env` 写成当前默认本地 secrets 路径；后端默认应优先看 `just sops-run`。
2. 不要把前端或桌面壳层启动步骤写成所有后端任务的默认前置条件。
3. 不要在这个 README 里重复维护所有 compose 细节，真实配置以脚本和 compose 文件为准。
