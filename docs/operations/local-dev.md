# Local Development

> 目的：说明本仓库当前后端默认本地开发入口，以及它如何对齐 `counter-service` reference chain。
>
> 本文档不试图覆盖所有前端或桌面运行方式；默认视角仍然是后端主链。

## 1. 核心结论

当前本地开发应优先理解为两层：

1. 基础依赖层：`infra/local/scripts/bootstrap.sh` 管理 NATS、Valkey、MinIO，以及可选的 sqld 相关基础设施。
2. 应用运行层：通过 `just` / `moon` 启动 `web-bff`、其他 BFF 或需要的开发进程。

后端默认入口不是 `.env` 驱动的全仓库教程，而是围绕 `counter-service` 主链建立的最小本地闭环。

## 2. 推荐阅读顺序

开始本地后端开发前，建议按以下顺序理解：

1. `docs/operations/counter-service-reference-chain.md`
2. `infra/local/README.md`
3. `justfiles/dev.just`
4. `justfiles/sops.just`
5. `infra/local/scripts/bootstrap.sh`

## 3. 当前真实入口

### 3.1 工具链准备

使用仓库已有命令：

```bash
just setup
just setup-deps
just doctor
just doctor-full
```

这些命令比手写安装步骤更接近当前仓库的真实入口。

### 3.2 启动基础依赖

当前本地基础设施入口是：

```bash
bash infra/local/scripts/bootstrap.sh up
bash infra/local/scripts/bootstrap.sh status
```

脚本当前会管理的核心依赖包括：

1. NATS
2. Valkey
3. MinIO
4. 可选的 Turso/libSQL client-server 模式相关端口信息

需要注意：

1. 默认业务路径仍主要使用嵌入式 libSQL/SQLite 形态。
2. sqld 是可选的本地实验路径，不应写成所有开发都必须依赖的默认前提。

### 3.3 启动后端开发进程

当前仓库真实存在的 just 入口包括：

```bash
just dev
just dev-web
just dev-api
just dev-admin-bff
```

其中：

1. `just dev-web` 对应 web 开发主路径。
2. `just dev-api` 是更贴近后端默认视角的入口之一。
3. 是否需要同时启动前端壳层，取决于当前任务，不应作为所有后端任务的默认要求。

### 3.4 本地 secrets 注入

当前后端默认不应把 `.env` 当成主路径。更符合当前仓库约束的方式是：

```bash
just sops-run DEPLOYABLE=web-bff ENV=dev
just sops-run DEPLOYABLE=outbox-relay-worker ENV=dev CMD='cargo run -p outbox-relay-worker'
just sops-run DEPLOYABLE=projector-worker ENV=dev CMD='cargo run -p projector-worker'
```

原因：

1. 这与集群中的 `SOPS -> Kustomize/Flux` 路径保持环境变量形状一致。
2. 可以避免本地路径和交付路径分叉得过早。

## 4. 以 counter 参考链理解本地开发

当前本地后端最值得优先跑通的不是“所有模块一起启动”，而是下面这条最小主线：

1. 启动本地基础依赖。
2. 启动 `web-bff` 或对应后端入口。
3. 必要时启动 `outbox-relay-worker`。
4. 需要验证 read model/replay 时，再启动 `projector-worker`。
5. 观察 counter 相关同步写入、outbox 写入和异步发布路径。

这样做的意义是：

1. 优先验证默认后端锚点。
2. 优先对齐后续服务应复用的工程路径。
3. 避免把还未收敛完成的外围模块当成默认学习入口。

## 5. 当前不应继续传播的旧说法

以下说法不再适合作为默认本地开发文档内容：

1. 要求所有后端开发都先复制 `.env.example` 到 `.env`。
2. 把前端和桌面启动流程写成后端任务的默认前置步骤。
3. 把尚未稳定的迁移、验证或运行命令写成已经收敛的统一入口。

例如：

1. `local-dev.md` 旧版里出现的 `.env` 主路径，与 `justfiles/sops.just` 的默认约束不一致。
2. 旧版中部分命令名称和当前 `justfiles/*.just` 的真实入口也并不完全对齐。

## 6. 验证建议

当前更贴近实际的本地验证入口是：

```bash
just test
just test-e2e
just validate-platform
just validate-deps
just gate-scoped service-agent
```

具体跑哪些验证，仍应按触达目录和 `agent/manifests/gate-matrix.yml` 决定。

## 7. 一句话结论

本仓库当前本地后端开发的默认主线，是围绕 `counter-service` 先跑通 `bootstrap -> web-bff -> relay -> 验证` 这条最小闭环，并尽量保持与 `SOPS/Kustomize/Flux` 工程路径一致，而不是回到 `.env` 驱动的传统教程式入口。
