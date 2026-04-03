# Requirements: Tauri-SvelteKit-Axum Boilerplate

**Defined:** 2026-04-01
**Milestone:** v0.2.0
**Core Value:** Agent-Native Cross-Platform Application Engineering Base — runnable, tested, production-ready with Google Auth, Counter, Admin Web, Agent conversation, contracts/typegen single-truth-source, and clear architectural boundaries.

## v0.2.0 Requirements

### 仓库结构与工具链

- [ ] **STRUCT-01**: 仓库目录结构对齐 agent-native-starter-v1 蓝图的 apps/servers/packages/crates/tools 分层，建立目录边界红线
- [ ] **TOOL-01**: moon + Just + proto 提供统一的 setup/dev/verify/typegen 入口，开发者可通过单条命令启动全栈开发和验证

### Contracts 与类型闭环

- [x] **CONTRACT-01**: packages/contracts/api 定义跨边界共享 DTO/contracts 作为 Rust 单一真理源
- [x] **CONTRACT-02**: typegen 从 Rust contracts 自动生成 TS 类型，CI 在生成后有未提交 diff 时失败

### Runtime 边界收敛

- [x] **RUNTIME-01**: core/domain 不依赖任何 host/protocol/chain，业务规则完全隔离
- [x] **RUNTIME-02**: adapters/hosts/tauri (runtime_tauri) 承载 Tauri command 桥接职责，native host 仅保留 builder/bootstrap
- [x] **RUNTIME-03**: 新增 capability 通过 feature 模块组合 core + contracts + adapters 实现，不绕过边界

### 最小功能实现

- [ ] **AUTH-01**: 用户可以通过 Google 账号登录，auth 通过 adapter 接入不污染 core
- [x] **COUNTER-01**: 用户可以使用计数器功能（increment/decrement/reset），验证前后端通信
- [x] **ADMIN-01**: 用户可以访问管理后台界面，包含基本统计卡片
- [x] **AGENT-01**: 用户可以通过导入 OpenAI 兼容的 API key 与产品内 agent 进行对话（桌面模式需 Tauri IPC 双路径）

### Agent-Friendly 开发基建

- [x] **AGENT-DEV-01**: .agents/ 目录包含 skills、prompts、playbooks、rubrics，agent 可读取规则、发现工具、执行任务、验证结果

## Future Requirements (Deferred)

### V2 高概率扩展

- **FUT-01**: Host adapter 体系做实（browser extension / miniapp host 实装）
- **FUT-02**: Worker replay / fixture 体系做实
- **FUT-03**: Offline sync / retry / reconnect 策略
- **FUT-04**: Tracing / otel sink 完整化
- **FUT-05**: Release automation 强化

### V3 实验边车

- **FUT-06**: HTTP/3 runtime lane
- **FUT-07**: ATProto / Farcaster / Nostr protocol adapter 骨架
- **FUT-08**: 复杂 multi-agent orchestration

## Out of Scope (This Milestone)

| Feature | Reason |
|---------|--------|
| HTTP/3 默认启用 | 实验边车，V3 候选 |
| 多协议 federation runtime | V2 能力 |
| 复杂多 agent 自主协作 | V3 候选 |
| Email/password auth | Google OAuth sufficient for V1 |
| Full RBAC | 基本 multi-tenancy only |
| 全栈技术栈替换 | 违反最小改动原则 |
| 新增重型基础设施 | 低 ROI for this milestone |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| STRUCT-01 | Phase 1 | Pending |
| TOOL-01 | Phase 1 | Pending |
| CONTRACT-01 | Phase 2 | Plan 02-01 complete |
| CONTRACT-02 | Phase 2/7 | Partial (generated types not consumed) |
| RUNTIME-01 | Phase 3 | Complete |
| RUNTIME-02 | Phase 3 | Complete |
| RUNTIME-03 | Phase 3 | Complete |
| AUTH-01 | Phase 4/6 | Pending (adapter wiring needed) |
| COUNTER-01 | Phase 4 | Complete |
| ADMIN-01 | Phase 4 | Complete |
| AGENT-01 | Phase 4/8 | Partial (Tauri IPC dual-path needed) |
| AGENT-DEV-01 | Phase 5/8 | Pending (prompts missing, no VERIFICATION.md) |

**Coverage:**
- v0.2.0 requirements: 12 total
- Mapped to phases: 12
- Unmapped: 0 ✓

---

*Requirements defined: 2026-04-01*
*Last updated: 2026-04-01 after v0.2.0 research phase*
