# Rust 生态技术选型 (2026年3月版)

> 激进纯度优先方案 - 风险与收益并存
> 基准日期: 2026-03-28

---

## 一、核心架构 (已确认)

| 层级 | 选型 | 版本 | 状态 |
|------|------|------|------|
| 桌面 shell | Tauri | 2.x | ✅ 稳定生产 |
| 前端框架 | SvelteKit | 2.x + Svelte 5 | ✅ Runes 稳定 |
| 后端 HTTP | Axum | 0.8.x | ✅ 生产验证 |
| 构建工具 | moon | 2.x | ✅ 已集成 |
| 包管理器 | bun | 1.3.x | ✅ 已配置 |

---

## 二、基础设施层 (激进替换)

### 2.1 隧道/反向代理

| 组件 | 传统方案 | Rust 替代 | 选型 | 风险 |
|------|----------|-----------|------|------|
| 公网穿透 | ngrok | **rathole** | ✅ rathole | 中 |
| 反向代理 | nginx | **Pingora** | ⚠️ 可选 | 高 |

- **rathole** (13K+ ⭐): 生产级 NAT 穿透，完全替代 ngrok
- **Pingora** (26K+ ⭐): Cloudflare 生产验证的 Rust 代理框架
- 建议开发环境用 rathole，生产环境保留 nginx (无完美 Rust 替代方案)

### 2.2 容器运行时

| 组件 | 传统方案 | Rust 替代 | 选型 | 风险 |
|------|----------|-----------|------|------|
| OCI 运行时 | runc | **youki** | ⚠️ 远期 | 高 |
| 容器管理 | docker | 脚本/youki | ⚠️ 远期 | 高 |

- **youki**: CNCF 沙箱项目，生产就绪 (7.3K ⭐)
- 短期保留 docker-compose 用于 nginx + 外部服务

### 2.3 数据存储 (激进方案)

| 组件 | 传统方案 | Rust 替代 | 选型 | 风险 |
|------|----------|-----------|------|------|
| 主数据库 | Postgres | **SurrealDB** / **Databend** | ⚠️ 评估中 | **极高** |
| KV/Cache | Redis | **Lux** / **redis-oxide** | ✅ Lux | 中 |
| 对象存储 | S3/MinIO | **Garage** | ✅ Garage | 低 |

- **SurrealDB**: 多模型 DB (文档+图+时序)，但数据模型变革大，需深度评估
- **Databend**: HTAP 云原生数仓，ClickHouse 替代
- **GreptimeDB**: 时序场景专用
- ⚠️ **选型建议**: 初期使用 libsql (已验证)，中期评估 SurrealDB
- **Lux** (185 ⭐): Redis 兼容，2-7x 性能，原生向量支持
- **Garage** (3.3K ⭐): S3 兼容，纯 Rust，轻量自托管

---

## 三、可观测性层 (Rust 全家桶)

### 3.1 日志/追踪

| 组件 | 传统方案 | Rust 替代 | 选型 | 风险 |
|------|----------|-----------|------|------|
| 日志管道 | Fluentd | **Vector** | ✅ Vector | 低 |
| 日志存储 | Elasticsearch | **OpenObserve** | ✅ OpenObserve | 低 |
| 追踪收集 | Jaeger | **Uptrace** | ⚠️ 可选 | 中 |

- **Vector** (21K ⭐): 最高性能可观测数据管道
- **OpenObserve**: 140x 低成本替代 Datadog/Splunk，单二进制部署
- 组合: Vector → OpenObserve (全栈 Rust)

### 3.2 指标

| 组件 | 传统方案 | Rust 替代 | 选型 | 风险 |
|------|----------|-----------|------|------|
| 时序数据库 | Prometheus | **GreptimeDB** | ⚠️ 评估 | 中 |
| 可视化 | Grafana | - | ⚠️ 保留 | - |

---

## 四、搜索/AI 存储层

| 组件 | 传统方案 | Rust 替代 | 选型 | 风险 |
|------|----------|-----------|------|------|
| 全文搜索 | Elasticsearch | **Meilisearch** | ✅ 可选 | 低 |
| 向量搜索 | Pinecone | **Qdrant** | ✅ 可选 | 低 |
| 嵌入存储 | - | **Turso** | ⚠️ 可选 | 中 |

- **Meilisearch** (68K ⭐): 全文搜索，Rust 原生，ML 集成
- **Qdrant** (25K ⭐): 向量搜索，Series B $50M 验证
- 建议: 初期跳过 AI 相关，中期按需引入

---

## 五、MCP / Dev Tools 层

### 5.1 代码索引 MCP

| 工具 | 用途 | 选型 | 状态 |
|------|------|------|------|
| 代码检索 | 代码库语义搜索 | **rust-analyzer-mcp** / **molaco/rust-code-mcp** | ✅ |
| 代码诊断 | 项目分析和转换 | **dexwritescode/rust-mcp** | ✅ |
| 文档搜索 | Crate 索引 | **cratedex** | ✅ |

### 5.2 Web 能力 MCP

| 工具 | 用途 | 选型 | 状态 |
|------|------|------|------|
| 网页搜索 | Exa AI / 官方搜索 | **@exadev/mcp-server-exa** | ✅ 已配 |
| 网页浏览 | 浏览器自动化 | **@強化 aset slime** | ✅ Chrome-DevTools |

### 5.3 领域 Skills

已有:
- Rust skills (179 rules, 14 categories)
- Svelte skills
- Frontend patterns
- Backend patterns
- Testing patterns
- PostgreSQL patterns
- Docker patterns
- API design
- Security review

---

## 六、风险评估矩阵

| 组件 | 替代激进程度 | 技术风险 | 维护风险 | 建议 |
|------|-------------|----------|----------|------|
| SurrealDB | ⚠️⚠️⚠️ | 极高 | 高 | 初期不采用 |
| youki | ⚠️⚠️⚠️ | 高 | 高 | v2+ 考虑 |
| Lux | ⚠️⚠️ | 中 | 中 | 生产验证后使用 |
| Garage | ✅ | 低 | 低 | 可直接采用 |
| rathole | ✅ | 低 | 低 | 推荐 |
| OpenObserve | ✅ | 低 | 低 | 推荐 |
| Vector | ✅ | 低 | 低 | 推荐 |
| Meilisearch | ✅ | 低 | 中 | 可选 |
| Qdrant | ✅ | 低 | 中 | AI 场景后期 |

---

## 七、最终选型建议 (v1)

```
开发环境:
├── rathole (隧道) ← 替代 ngrok
├── lux/embedded-redis (缓存) ← 替代 redis 容器
└── libsql (数据库) ← 嵌入式，无需容器

生产环境:
├── nginx (反向代理) ← 暂无可替代方案
├── Redis (缓存) ← 生产级
└── SurrealDB/Databend ← 评估后决定 (非 v1)
```

### v1 基础设施依赖 (保守)

| 依赖 | 方案 | 原因 |
|------|------|------|
| Cache | redis-rs 客户端 + docker-redis | Lux 生产验证中 |
| DB | libsql / tauri-plugin-libsql | 嵌入式，简单 |
| Tunnel | rathole | Rust 原生，成熟 |
| Proxy | nginx | 无 Rust 完美替代 |
| S3 | Garage | 轻量自托管，可选 |
| Observability | Vector + OpenObserve | 全栈 Rust，推荐 |

### v2 候选升级

- SurrealDB 3.0 稳定后评估
- youki 成熟后考虑
- Qdrant (AI 搜索场景)

---

## 参考资料

### 官方/研究来源
- Tauri Blog: https://v2.tauri.app/blog/tauri-20/
- rust-docs.md (TWiR 2024-09 ~ 2026-03)
- This Week in Rust (108 projects)

### 关键第三方验证
- Cloudflare Blog: Pingora 生产验证
- Qdrant Series B $50M (2026-03)
- CNCF: youki 沙箱批准 (2024-10)
- SurrealDB 3.0 benchmarks

### 搜索来源
- Exa Code API (实时)
- Context7 docs
- crates.io最新版本
- GitHub stars/最新活动

---

*Last updated: 2026-03-28*
*Maintainer: AI Research Agent*
*Review cycle: 每季度*