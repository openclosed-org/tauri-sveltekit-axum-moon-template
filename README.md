# tauri-sveltekit-axum-moon-template

Tauri 2 + SvelteKit + Axum 跨端 monorepo 模板。
支持单 VPS → K3s → 微服务拓扑无缝切换。

## Agent 必读文档

| 文档 | 用途 |
|------|------|
| [AGENTS.md](AGENTS.md) | AI 协作协议与工程规则 |
| [docs/architecture/repo-layout.md](docs/architecture/repo-layout.md) | 目录布局规则与硬约束 |
| [agent/codemap.yml](agent/codemap.yml) | 模块约束与依赖规则 |

## 快速开始

```bash
just --list    # 查看所有命令
mise doctor    # 工具链检查
```

## 核心原则

1. **平台模型优先** — `platform/model/*` 是真理源，一切由其生成
2. **契约先于实现** — `packages/contracts/*` 先改，再改实现
3. **Services 是库，不是进程** — 可被 servers/workers 同时复用
4. **Workers 是一等公民** — 所有异步执行单元在 `workers/`
5. **Vendor 只能进 adapters** — 具体中间件 SDK 只在 `packages/*/adapters/`
6. **生成物禁止手改** — sdk/rendered/catalog 必须可删可再生
7. **拓扑切换靠 topology model，不靠重构** — `platform/model/topologies/*.yaml`
