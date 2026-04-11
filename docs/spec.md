# 📜 架构宪法：设计哲学 · 决策原则 · 目标与约束

> **核心信念**：架构不是"设计出来的"，而是"约束出来的"。
> 通过 **目录职责注释 + 依赖方向规则 + 工具链决策矩阵 + 验证命令** 四层约束，
> 所有 100% 会遇到的关键决策现在一次性明确，后续只执行不讨论，只验证不修改。

---

## 🎯 项目目标

### 愿景

构建一个 **Tauri 2 + SvelteKit + Axum** 跨端 monorepo 模板，支持：
- **中心化体验**：强一致的用户/租户管理、高频业务逻辑、SSR 友好的 Web 端
- **去中心化资产/身份**：Web3 协议集成（Nostr/Farcaster/Base/Solana/TON）、密码学身份、链上状态索引
- **本地优先**：OfflineFirst 同步策略、Turso embedded replica、断网可用
- **模块化单体**：独立 Cargo crate + 独立数据库 schema + 独立契约，支持未来物理拆分

### 成功标准

| 维度 | 量化指标 | 验证方式 |
|------|----------|----------|
| **可维护性** | 新模块生成到 CI 通过 ≤ 5 分钟 | `just quality boundary` 零违规 |
| **类型安全** | 前端类型错误 = 契约不兼容 | `just gen-frontend-sdk && bun build` |
| **同步可靠** | embedded 命中率 > 80%，冲突率 < 1% | OpenObserve dashboard |
| **部署效率** | 一键启动开发环境 | `just deploy compose` |
| **Agent 友好** | Agent 生成代码必须通过 CI | `just ci-check-contracts` |

---

## 🧭 架构哲学

### 核心原则

1. **最小改动、最小认知成本、可验证结果**
   - 以交付为目标，避免过度设计
   - 先做最小可验证闭环，再逐步完善

2. **先证据后判断，先搜索后猜测**
   - 遇到报错，先获取完整日志、复现步骤、版本信息
   - 涉及新框架/新版本/陌生 API，优先查官方文档/Issues/Release Notes

3. **先小后大，局部改动**
   - 优先最小闭环、局部化改动、可回滚方案
   - 宁可小范围重复，也不要过早抽象

4. **契约优先，显式不变量**
   - 所有跨层/跨服务交互必须通过 `contracts/` 定义的契约
   - 不变量必须在编译期或 CI 中显式校验

5. **未来 agent 可持续编辑**
   - 所有决策必须文档化，所有约束必须可验证
   - Agent 生成代码必须在约束边界内，CI 必须拦截越界行为

### 技术决策默认优先级

1. 满足需求与验收标准
2. 正确性与边界条件
3. 回归风险与改动范围
4. 复用现有模式与项目约定
5. 可测试性与可维护性
6. 交付速度
7. 扩展性与性能优化

---

## 🏗️ 架构偏置

### 依赖方向（不可违反）

```
contracts/     ← 所有共享类型的单一真理源
features/      ← 定义 trait + 类型，不得包含实现，不得依赖 usecases
usecases/      ← 实现 features 定义的 trait，依赖 domain + features + contracts
adapters/      ← 外部世界翻译层，不得承载业务逻辑
apps/ / servers/ ← 组合层，不得包含业务逻辑
```

### 分层职责

- **Tauri command / Axum handler**：主要承担适配与协议职责
- **核心业务逻辑**：进入稳定、可测试的 `usecases/`（对应 `services/*/application/`）
- **SvelteKit**：负责 UI 组合与瞬时状态，不承载领域真相
- **跨 Rust / TypeScript 边界**：优先 typed/shared contracts（由 `contracts/` 生成）

### 架构桥接原则

- 优先 vertical slice 和局部修改
- 宁可小范围重复，也不要过早抽象
- 同一业务概念在不同层的类型必须对齐，不得出现字段级差异
- **禁止过度设计**：先做最小可验证闭环

---

## 🌐 Web3 / 去中心化协议集成原则

### 核心原则：协议即适配器，状态显式路由，契约双向校验

传统云原生与 Web3/去中心化协议并非"互斥"，而是**解决不同维度的问题**。冲突只发生在"设计哲学混用"时（例如用中心化 DB 存私钥、用智能合约做高频交易、用 JWT 链上验签）。只要通过 **Trait 抽象 + 显式数据路由 + 独立 CI/CD 路径**，它们不仅能共存，还能形成"中心化体验 + 去中心化资产/身份"的混合架构。

### 为什么"看似冲突" vs "实际互补"

| 维度 | 传统云原生/微服务 | Web3/去中心化协议 | 冲突点（表面） | 兼容点（本质） |
|------|------------------|-------------------|----------------|----------------|
| **身份模型** | OAuth/JWT/Session | 密码学私钥/钱包/Nostr PubKey/Farcaster Signer | "谁控制身份？" | 可抽象为 `AuthAdapter`，映射到统一 `UserId` |
| **状态所有权** | 服务端集中管理 | 用户自托管/链上账户/中继网络 | "数据归谁？" | 可路由：体验数据→中心DB，资产/凭证→链上/协议 |
| **一致性模型** | 强一致/ACID | 最终一致/概率确定/中继 gossip | "写冲突怎么办？" | 可声明 `SyncStrategy`：OnlineOnly / Eventual / OnChainFinality |
| **通信拓扑** | Client→Server→MQ | P2P/Relay/Hub/Gossip/跨链桥 | "消息怎么传？" | 可统一为 `EventBus` trait，按协议切换实现 |
| **部署与升级** | GitOps/K8s/滚动发布 | 智能合约不可变/中继集群/链下索引 | "怎么更新？" | 可分离 CI：中心化走 Flux，链上走 Anchor/Tact/Deploy 脚本 |

✅ **结论**：冲突不在技术栈，而在**数据流向与信任边界未显式定义**。只要提前划定"什么走中心化、什么走去中心化、什么可双向同步"，Monorepo 完全可承载。

### 架构铁律（违反必崩）

1. **私钥永不离开客户端/安全 enclave**  
   `services/` 和 `packages/adapters/` 只能做签名验证或已签名交易广播，绝不经手明文私钥。

2. **链上状态 ≠ 业务状态**  
   智能合约/链上账户仅作为"最终事实源"，业务查询走索引器 → Turso 读优化。禁止直接链上 `call` 做高频查询。

3. **一致性模型必须显式声明**  
   任何涉及去中心化协议的数据写入，必须在 `SyncStrategy` 中新增 `OnChainFinality` 或 `EventualRelay`，并在 `contracts/` 中声明超时/重试/回滚语义。

4. **CI/CD 严格分离但统一编排**  
   中心化服务走 `Flux + K8s`，链上程序/中继部署走独立 workflow。Monorepo 通过 `moon affected` 控制触发范围，避免误触链上部署。

5. **协议集成默认 `stub`，按需启用 `feature`**  
   ```toml
   # packages/web3/Cargo.toml
   [features]
   default = []
   nostr = ["dep:nostr-sdk", "dep:tokio"]
   farcaster = ["dep:farcaster-hub-client"]
   base = ["dep:alloy", "dep:tokio"]
   # 业务模块仅依赖 traits，协议实现通过 feature 注入
   ```

### 何时绝对不要混用（红线）

| 场景 | 为什么危险 | 正确做法 |
|------|------------|----------|
| 用中心化 DB 存储链上交易最终结果 | 信任模型破坏，审计无法自证 | 链上状态仅作为权威源，DB 仅做读缓存 |
| 在 Axum 路由中直接解析 Nostr 事件/TON Cell | 耦合协议细节，升级即重构 | 通过 `indexer/` 标准化为业务 DTO |
| 用 JWT 模拟钱包登录 | 身份模型错位，无法链上交互 | 统一走 `IdentityProvider` trait，前端签名 → 后端验签 |
| 在同一个 PR 中改 K8s 部署 + 升级 Solana 程序 | 回滚灾难，故障域交叉 | 分离 CI pipeline，链上升级走独立审批流 |
| 期望去中心化协议提供 SLA/强一致 | 范式错误 | 明确声明 `Eventual/Probabilistic`，业务层做降级/补偿 |

---

## 🔧 工具链选型决策

### 核心原则

```
✅ 必须提前确定: 所有工具链选型必须现在决策, 后续只执行不讨论
✅ 必须文档化: 每个决策必须有 ADR + 验证命令 + 回滚方案
✅ 必须可验证: 每个工具必须有 `--version` + 健康检查命令
✅ 必须可替换: 通过 feature flag / adapter pattern 支持未来替换
```

### 工具链决策表

| 类别          | 工具                   | 版本约束             | 选型理由                                             | 替代方案                     | 风险                                | 验证命令                                                            | 回滚方案                             |
| ------------- | ---------------------- | -------------------- | ---------------------------------------------------- | ---------------------------- | ----------------------------------- | ------------------------------------------------------------------- | ------------------------------------ |
| **版本管理**  | mise                   | >=2024.1             | ✅ 统一管 Rust/Node/bun/moon/just/biome，支持 mise.toml 声明式 | asdf + 多插件                | ⚠️ 实验性功能需 MISE_EXPERIMENTAL=1 | `mise --version` + `mise doctor`                                    | 回退到 .tool-versions + asdf         |
| **任务编排**  | moon                   | >=1.0                | ✅ Rust 编写, 依赖感知构建, affected 计算精准        | turborepo (JS) / just 纯脚本 | ⚠️ 学习曲线, 配置复杂               | `moon --version` + `moon ci:affected --base=main`                   | 回退到 just + 手动 affected 计算     |
| **命令入口**  | just                   | >=1.0                | ✅ 跨平台, 语法简洁, Agent 友好, 薄层转发到 moon     | make / npm scripts           | ⚠️ 复杂逻辑仍需 shell               | `just --version` + `just --list`                                    | 回退到 shell scripts + README 命令   |
| **前端工具**  | bun                    | >=1.3                | ✅ monorepo 支持最佳, 磁盘高效, 速度快               | npm / pnpm                   | ⚠️ 部分包兼容性问题                 | `bun --version` + `bun --filter '*' echo $PWD`                      | 回退到 npm workspaces                |
| **前端 Lint** | biome                  | >=1.0                | ✅ 速度快, Svelte 支持好, monorepo 友好              | eslint + prettier            | ⚠️ 规则迁移成本                     | `bun exec biome check .`                                            | 回退到 eslint + prettier             |
| **Rust Lint** | clippy                 | stable               | ✅ 官方, 规则丰富, 可配置                            | cargo-deny (仅依赖)          | ⚠️ 误报需配置忽略                   | `cargo clippy --version` + `cargo clippy -- -D warnings`            | 调整 clippy.toml 忽略规则            |
| **依赖审计**  | cargo-deny             | >=0.14               | ✅ 许可证/安全/来源多维审计                          | cargo-audit (仅安全)         | ⚠️ 配置复杂                         | `cargo deny --version` + `cargo deny check`                         | 回退到手动审查 + cargo-audit         |
| **依赖统一**  | cargo-hakari           | >=0.9                | ✅ 自动同步 workspace 依赖版本                       | 手动维护                     | ⚠️ 首次配置复杂                     | `cargo hakari --version` + `cargo hakari generate`                  | 回退到手动同步 + cargo tree 校验     |
| **测试框架**  | nextest                | >=0.9                | ✅ 并行执行, 输出友好, 支持 coverage                 | cargo test                   | ⚠️ 配置略复杂                       | `cargo nextest --version` + `cargo nextest run`                     | 回退到 cargo test                    |
| **覆盖率**    | llvm-cov               | >=0.5                | ✅ 与 nextest 集成好, 输出 HTML                      | tarpaulin                    | ⚠️ 编译慢                           | `cargo llvm-cov --version` + `cargo llvm-cov --html`                | 回退到 tarpaulin                     |
| **集成测试**  | testcontainers-rs      | >=0.18               | ✅ Rust 原生, 支持 Turso/NATS/Redis                  | 手动 docker-compose          | ⚠️ 启动慢                           | `cargo test -p user -- integration`                                 | 回退到手动启动依赖 + 测试            |
| **契约生成**  | utoipa                 | >=4.0                | ✅ Axum 集成好, 类型安全, 支持 OpenAPI 3.1           | paperclip / apistos          | ⚠️ 学习曲线                         | `just gen-openapi` + `git diff --exit-code openapi.snapshot.json`   | 回退到手动维护 OpenAPI YAML          |
| **类型同步**  | ts-rs                  | >=0.9                | ✅ Rust→TS 类型同步, 编译期校验                      | manual typing                | ⚠️ 配置复杂                         | `just gen-frontend-sdk` + 前端编译校验                              | 回退到手动维护 typescript 类型       |
| **事件契约**  | AsyncAPI + JSON Schema | 3.1 + 2020-12        | ✅ 语言无关, Agent 友好, 校验严格                    | 自定义 YAML                  | ⚠️ 工具链分散                       | `just gen-asyncapi` + `ajv validate -s schema.json -d payload.json` | 回退到手动维护事件文档               |
| **数据库**    | turso                  | >=0.4                | ✅ Rust 重写, embedded replica, 离线同步             | libSQL (SQLite fork)         | ⚠️ concurrent writes 仍 beta        | `cargo tree -p adapter-turso` + `cargo test -p adapter-turso`       | 回退到 libSQL + 手动同步逻辑         |
| **复杂模型**  | surrealdb              | >=3.0                | ✅ 多模型 (图/向量/时序), 可选启用                   | neo4j + elasticsearch        | ⚠️ 资源占用高, 学习曲线             | `cargo build -p adapter-surreal --features experimental`            | 保持 feature = "experimental" 不启用 |
| **缓存**      | moka                   | >=0.12               | ✅ 零外部依赖, 高性能, async 友好                    | redis + redis-rs             | ⚠️ 内存占用 (可配置)                | `cargo test -p adapter-cache`                                       | 启用 valkey feature + 配置 Redis     |
| **消息队列**  | async-nats             | >=0.33               | ✅ Rust 原生, JetStream 支持, 轻量                   | kafka + rdkafka              | ⚠️ NATS 集群运维                    | `cargo test -p event-bus --features nats`                           | 保持 feature = "memory" 默认         |
| **可观测**    | opentelemetry-rust     | >=0.22               | ✅ 标准协议, Axum 集成好, 动态采样                   | 自定义 tracing               | ⚠️ 配置复杂                         | `just deploy compose` + 检查 Jaeger UI                              | 回退到 tracing + 文件日志            |
| **日志聚合**  | vector                 | >=0.35               | ✅ Rust 原生, 高性能, 转换灵活                       | fluentd / logstash           | ⚠️ 配置 YAML 复杂                   | `vector --version` + `vector validate config.yaml`                  | 回退到文件日志 + 手动分析            |
| **指标存储**  | openobserve            | >=0.9                | ✅ 统一日志/指标/追踪, 成本低, Rust 编写             | prometheus + grafana + loki  | ⚠️ 社区较小                         | `docker compose -f observability.yaml up` + 访问 UI                 | 回退到 Prometheus + Grafana + Loki   |
| **部署编排**  | K3s + Flux             | K3s>=1.28, Flux>=2.0 | ✅ 轻量 K8s + GitOps, 生产验证                       | k0s + ArgoCD                 | ⚠️ K3s 功能裁剪                     | `k3s --version` + `flux --version` + `just deploy k8s-diff`         | 回退到 docker-compose + systemd      |
| **服务网格**  | linkerd                | >=2.14               | ✅ 轻量, mTLS/重试/限流零代码修改                    | istio                        | ⚠️ 学习曲线                         | `linkerd --version` + `linkerd check`                               | 保持未注入, 用应用层重试逻辑         |
| **密钥管理**  | sops + age             | sops>=3.8, age>=1.1  | ✅ 声明式加密, 多环境隔离, Git 友好                  | vault / aws secrets          | ⚠️ 密钥轮换流程                     | `sops --version` + `age --version` + `sops decrypt test.sops.yaml`  | 回退到 .env + 手动密钥管理           |
| **策略引擎**  | regorus                | >=0.1                | ✅ OPA 兼容, Rust 嵌入, 零网络开销                   | casbin-rs                    | ⚠️ Rego 语言学习                    | `regorus --version` + `regorus test policy.rego`                    | 回退到应用层硬编码策略               |
| **备份工具**  | rustic                 | >=0.7                | ✅ restic 兼容, Rust 原生, 加密增量                  | kopia / restic               | ⚠️ 社区较小                         | `rustic --version` + `rustic backup --dry-run`                      | 回退到手动 rsync + 加密              |

---

## 📋 存储策略与同步语义

### 数据分类与存储路由

| 数据类型 | 示例 | 存储方案 | 同步策略 | 冲突解决 | 位置 |
|---------|------|---------|---------|---------|------|
| 🎨 应用配置 | 主题/语言/偏好 | Tauri Store | 无 (纯本地) | 无 | 本地文件系统 (~/.app/settings.json) |
| 👤 用户私有数据 | 草稿/个人设置 | Turso Embedded | OfflineFirst | ClientWins | local.db + 云端备份 |
| 💼 租户业务数据 | counter/订单 | Turso Embedded + Cloud | OfflineFirst | LWW + 业务规则 | local.db ↔ Turso Cloud |
| 🌐 共享/公共数据 | 产品目录/公告 | Turso Cloud Only | OnlineOnly | ServerWins | Turso Cloud |
| 📊 分析/日志数据 | 点击流/错误日志 | Turso QueueOnly | QueueOnly | 无 (追加) | local queue → batch sync |
| 🧠 复杂关系数据 | 社交图/推荐 | SurrealDB (可选) | OnlineOnly | Custom | SurrealDB Cluster |
| 🔗 链上状态/协议事件 | 交易哈希/中继事件/索引数据 | Indexer → Turso 读优化 | OnChainFinality / EventualRelay | 链上权威 / 多中继共识 | 链上 + 本地索引缓存 |

### 同步策略选择规则（编译期强制）

```rust
// 所有 repo 方法签名必须包含 (不可省略):
pub async fn increment(
    &self,
    ctx: &TenantContext,      // ✅ 编译期防租户泄漏
    strategy: SyncStrategy,   // ✅ 编译期强制声明同步语义
    counter_id: CounterId,
) -> Result<Counter>;

// SyncStrategy 枚举 (不可扩展, 必须提前确定):
pub enum SyncStrategy {
    OfflineFirst,     // 本地先写, 后台异步同步 (默认)
    OnlineOnly,       // 必须在线, 直接写云端
    QueueOnly,        // 离线只入队, 联网后批量同步
    OnChainFinality,  // 链上交易确认, 不可逆
    EventualRelay,    // 去中心化中继网络, 最终一致
}
```

### 冲突解决策略

```rust
// ConflictResolver trait (不可更改接口)
#[async_trait]
pub trait ConflictResolver: Send + Sync {
    async fn resolve(
        &self,
        local: RecordVersion,
        remote: RecordVersion,
    ) -> Result<Resolution, ConflictError>;
}

// 内置策略 (必须实现, 不可新增)
pub enum BuiltInResolver {
    Lww,         // Last-Write-Wins (基于 timestamp)
    ClientWins,  // 本地覆盖远程 (用户私有数据)
    ServerWins,  // 远程覆盖本地 (共享/平台数据)
}

// 自定义策略 (必须通过 ADR 审批 + 完整测试)
// 禁止: 业务模块直接实现复杂冲突逻辑 (必须复用内置或审批后自定义)
```

### 验证命令

```bash
# 存储策略校验 (CI 必须执行)
$ just verify-storage-policy
# → 校验所有 repo 方法是否声明 SyncStrategy
# → 校验 ConflictResolver 实现是否覆盖测试场景

# 同步功能测试 (开发/发布必须执行)
$ cargo test -p counter -- sync
# → 必须包含: offline_write + concurrent_edit + conflict_resolution

# 云费监控 (生产必须启用)
# OpenObserve dashboard 必须包含:
# - sync.pushed_bytes / sync.pulled_bytes
# - sync.conflict_rate (告警阈值: >1%)
# - turso.embedded_hits (目标: >80%)
```

---

## 🔒 依赖方向与模块边界

### 依赖方向规则（编译期 + CI 双重校验）

✅ **允许**:
- `apps/*` → `packages/contracts/sdk` + `packages/ui`
- `services/*` → `packages/core` + `packages/contracts`
- `packages/adapters/*` → `packages/core` + external crates
- `bff/*` → `services/*` (via trait) + `packages/contracts`

❌ **禁止** (CI 拦截):
- `apps/*` → `services/*` (必须通过 BFF)
- `services/*` → `apps/*` (循环依赖)
- `services/*` → other `services/*` (必须通过 events/contracts)
- `packages/core` → any external crate except serde/thiserror/uuid/chrono

### 模块边界定义（moon 校验）

```yaml
# moon.yml 中的依赖约束 (必须现在确定)
projects:
  counter:
    path: services/counter
    depends_on:
      - packages/core/kernel
      - packages/core/platform
      - packages/contracts
    forbidden_dependencies:
      - axum
      - tokio
      - sqlx
      - surrealdb
    allowed_crates:
      - serde
      - uuid
      - chrono
      - thiserror
      - tracing
```

### 验证命令

```bash
# 依赖方向校验 (CI 必须执行)
$ just quality boundary
# → 使用 cargo-modules 或自定义脚本校验依赖图
# → 循环依赖/越界依赖直接失败

# 模块独立构建 (发布前必须验证)
$ cargo build -p counter --workspace
# → 必须可独立编译, 不依赖其他 services/*

# 物理拆分预演 (架构演进前必须验证)
$ ./ops/scripts/preview-split.sh counter
# → 模拟将 counter 拆为独立 repo, 验证配置/契约/部署是否就绪
```

---

## 📜 契约生成与版本管理

### 契约变更流程（不可绕过）

1. **修改 contracts/ 前**:
   - 必须更新 `docs/contracts/changelog.md`
   - 必须标注 `[BREAKING]` if 接口不兼容
   - 必须通知所有依赖方 (frontend/other services)

2. **生成新 snapshot**:
   - `just gen-openapi` → 更新 `openapi.snapshot.json`
   - `just gen-asyncapi` → 更新 `asyncapi.yaml`
   - `just gen-frontend-sdk` → 更新 `packages/sdk/typescript`

3. **CI 校验**:
   - `git diff --exit-code docs/contracts/http/openapi.snapshot.json`
   - 前端编译必须通过 (类型安全校验)
   - contract tests 必须通过 (序列化兼容)

4. **合并后**:
   - 必须更新 `agent/checklists/schema-change.md`
   - 必须通知 Agent 更新 `codemap.yml` (如有边界变更)

### 版本管理规则

```yaml
# OpenAPI/AsyncAPI 版本策略
- HTTP API: 使用 path versioning (/v1/counter), 不兼容变更必须新增版本
- Events: 使用 subject versioning (counter.v1.incremented), 不兼容变更必须新增 subject
- DTO: 使用 JSON Schema $id + $schema, 不兼容变更必须新增 $id

# 前端 SDK 同步策略
- 必须通过 packages/contracts/sdk-gen 自动生成
- 禁止手写 typescript 类型 (必须引用 generated/)
- 前端编译失败 = 契约不兼容 = 必须修复后端或协调前端
```

### 验证命令

```bash
# 契约校验 (CI 必须执行)
$ just ci-check-contracts
# → 生成 snapshot + 比对 diff + 运行 contract tests

# 前端类型安全校验 (PR 必须通过)
$ bun --filter 'web' run build
# → 必须使用 generated SDK, 类型错误直接失败

# 事件契约校验 (发布前必须执行)
$ just verify-asyncapi
# → 校验 asyncapi.yaml + JSON Schema + 示例 payload
```

---

## ✅ 最终验证清单（架构就绪标准）

```bash
# 目录结构验证
$ find . -name ".gitkeep" | wc -l  # 必须 > 200 (确保所有目录被追踪)
$ just quality boundary            # 依赖方向必须零违规
$ moon ci:affected --base=main     # affected 计算必须准确

# 工具链验证
$ mise doctor                      # 所有工具版本必须匹配
$ just --list                      # 所有命令必须可用
$ cargo hack check --workspace     # 依赖必须无冲突

# 契约验证
$ just gen-openapi && git diff --exit-code  # OpenAPI 必须稳定
$ just gen-frontend-sdk && bun --filter 'web' build  # 前端必须类型安全
$ just verify-asyncapi            # 事件契约必须有效

# 存储策略验证
$ cargo test -p counter -- sync   # 同步测试必须 100% 通过
$ just verify-storage-policy      # 存储策略约束必须满足
$ docker compose -f app.yaml up   # 开发环境必须一键启动

# Agent Harness 验证
$ agent "add new field to Counter"  # 必须自动生成 + 通过 CI
$ agent "call sqlx directly"        # 必须被 CI 拦截
$ agent "modify contracts without changelog"  # 必须被 checklist 拦截

# 演进路径验证
$ ./ops/scripts/preview-split.sh counter  # 物理拆分预演必须成功
$ cargo build -p counter --release        # 独立构建必须成功
$ just deploy compose -s counter          # 独立部署必须成功
```

---

## 🚀 快速开始

```bash
git clone <this-template>
just setup                    # 安装工具链
just verify-architecture      # 验证架构约束
just agent init              # 初始化 Agent 约束
just dev up                  # 启动开发环境
# 开始用 Agent 生成你的第一个模块
```
