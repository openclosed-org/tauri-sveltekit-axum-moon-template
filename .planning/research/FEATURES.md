# Feature Research

**Domain:** Agent-Native Cross-Platform Application Engineering Base
**Researched:** 2026-04-01
**Confidence:** HIGH (sourced from blueprint minimum implementation scope)

## Feature Landscape

### Table Stakes (Users Expect These)

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| Google Auth | V1 最小实现范围，用户登录入口 | MEDIUM | 通过 adapter 接入，不污染 core |
| Counter | 验证前后端通信的基础功能 | LOW | 通过 feature 组合 core + contracts 实现 |
| Admin Web | 管理后台界面 | MEDIUM | 通过 feature + UI 组件实现 |
| Agent 对话 | 用户导入 API key 与 agent 对话 | HIGH | OpenAI 兼容 API，Rust 轻量级 agent 框架 |

### Differentiators (Competitive Advantage)

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| Contracts/typegen 单一真理源 | 避免 agent 开发时四套真相漂移 | HIGH | Rust→TS 自动生成，CI drift 检查 |
| Agent-Friendly 开发基建 | 让 agent 能高效开发和迭代 | MEDIUM | AGENTS/skills/playbooks/rubrics |
| 跨宿主 adapter 体系 | 一次业务逻辑，多宿主运行 | HIGH | Tauri/browser extension/miniapp |
| 三层稳定性模型 | 永久核心 / 高概率扩展 / 实验边车 | LOW | 架构层面的可演进性 |

### Anti-Features (Commonly Requested, Often Problematic)

| Feature | Why Requested | Why Problematic | Alternative |
|---------|---------------|-----------------|-------------|
| 实时同步所有数据 | "用户需要实时体验" | 复杂度爆炸，性能问题 | 按需实时，关键路径才上 |
| 过度切分 crate | "微服务架构好" | 编译时间增加，依赖关系复杂 | 只在复用或编译边界明显时拆 |
| 默认启用 HTTP/3 | "前沿技术" | 实验性，复杂度高 | V3 候选，不进默认基线 |
| 复杂 RBAC | "企业级需求" | 过度设计，增加复杂度 | 基本 multi-tenancy only for V1 |

## Feature Dependencies

```
[Contracts/typegen 闭环]
    └──requires──> [目录结构对齐蓝图]
                       └──requires──> [工具链任务图补全]

[Google Auth] ──requires──> [Auth adapter]
[Counter] ──requires──> [Contracts + Core domain]
[Admin Web] ──requires──> [Feature + UI components]
[Agent 对话] ──requires──> [Contracts + async-openai + Rust agent 框架]
```

## MVP Definition

### Launch With (v0.2.0)

- [ ] 仓库目录结构对齐蓝图
- [ ] Contracts/typegen 闭环
- [ ] 工具链任务图（moon + Just + proto）
- [ ] Google Auth
- [ ] Counter
- [ ] Admin Web
- [ ] Agent 对话
- [ ] Agent-Friendly 基建（AGENTS/skills/playbooks/rubrics）

### Future (V2+)

- [ ] Host adapter 体系做实（browser extension / miniapp）
- [ ] Worker replay / fixture 体系
- [ ] Offline sync / retry / reconnect
- [ ] HTTP/3 runtime lane
- [ ] ATProto / Farcaster protocol adapters

## Sources

- docs/blueprints/agent-native-starter-v1/00-index.md — V1 minimum implementation scope
- docs/blueprints/agent-native-starter-v1/10-roadmap-v1-v3.md — V1/V2/V3 scope
- docs/blueprints/agent-native-starter-v1/05-runtime-features-and-adapters.md — Feature structure

---
*Feature research for: v0.2.0 架构蓝图对齐与核心功能实现*
*Researched: 2026-04-01*
