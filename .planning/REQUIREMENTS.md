# Requirements: geek taste

**Defined:** 2026-03-22
**Core Value:** 高信噪比、可行动、低打扰的技术信号

## v1 Requirements

Requirements for initial release. Each maps to roadmap phases.

### Foundation

- [ ] **FOUND-01**: 用户能输入 GitHub PAT 并保存到 OS 安全存储
- [ ] **FOUND-02**: 用户能查看仓库基本信息（stars, forks, description, language, topics）
- [ ] **FOUND-03**: 用户能配置通知频率、语言兴趣、安静时段等设置
- [ ] **FOUND-04**: 应用提供 Home/TopK/Subscriptions/Resources/Rules 导航壳与路由

### TopK Discovery

- [ ] **TOPK-01**: 用户能按 STARS_DESC 排序查看趋势仓库
- [ ] **TOPK-02**: 用户能按 UPDATED_DESC 排序查看最近更新的仓库
- [ ] **TOPK-03**: 系统每 12h 自动创建排名快照，支持时间对比
- [ ] **TOPK-04**: 系统支持 Momentum 评分公式（0.5×star_delta + 0.2×fork_delta + 0.3×recency）
- [ ] **TOPK-05**: 用户能保存多个过滤+排序组合为可复用的 RankingView
- [ ] **TOPK-06**: 用户能按 language/topic/stars 阈值筛选候选集
- [ ] **TOPK-07**: 榜单变化时系统生成 VIEW_CHANGED 信号
- [ ] **TOPK-08**: 用户能从 TopK 榜单项一键订阅仓库

### Subscriptions & Signals

- [ ] **SUB-01**: 用户能搜索仓库并创建订阅
- [ ] **SUB-02**: 用户能编辑/暂停/删除已有订阅
- [ ] **SUB-03**: 系统轮询已订阅仓库的 Releases 和 Tags 生成信号
- [ ] **SUB-04**: 系统按 U1-U4 优先级规则去重（RELEASE_PUBLISHED > TAG_PUBLISHED > DEFAULT_BRANCH_ACTIVITY_DIGEST）
- [ ] **SUB-05**: 系统生成 12h/24h digest 窗口内的聚合信号
- [ ] **SUB-06**: HIGH 优先级信号触发桌面通知
- [ ] **SUB-07**: 用户能标记信号为已读/已处理
- [ ] **SUB-08**: 同一信号不会重复生成（幂等同步）

### Resource Radar

- [ ] **RES-01**: 系统能按语言/框架发现 MCP/Skills/Agent 资源
- [ ] **RES-02**: 系统按 stack_relevance + star_delta + recency 为资源评分
- [ ] **RES-03**: 每条资源推荐展示"为什么推荐给我"

### Home & Offline

- [ ] **HOME-01**: Home 页面聚合自上次访问以来的高优先级信号摘要
- [ ] **HOME-02**: 信号按优先级 + 时间 + 来源类型 + 用户亲和度排序
- [ ] **HOME-03**: 应用可离线打开并展示上次同步缓存，标记 STALE 状态

### Infrastructure

- [ ] **INFRA-01**: GitHub API 速率预算按 core/search 端点隔离管理
- [ ] **INFRA-02**: 同步逻辑支持 ETag 条件请求和增量刷新
- [ ] **INFRA-03**: SQLite 使用 WAL 模式，支持并发读写

## v2 Requirements

Deferred to future release. Tracked but not in current roadmap.

### Notifications

- **NOTF-01**: MEDIUM 优先级信号进入 digest（非即时通知）
- **NOTF-02**: 用户可配置每个订阅的通知级别
- **NOTF-03**: 安静时段自动抑制非 HIGH 通知

### Advanced Features

- **ADV-01**: PR_MERGED_DIGEST 高级模式
- **ADV-02**: TopK Momentum 暖机降级（无快照时回退 UPDATED_DESC 并提示）
- **ADV-03**: 订阅首次同步建立 baseline（不回溯全部历史）

## Out of Scope

| Feature | Reason |
|---------|--------|
| GitHub 全站搜索替代品 | 不与 GitHub 本体竞争 |
| Issues/PR/Discussions 全功能收件箱 | 范围过大，偏离信号聚焦 |
| 秒级实时告警系统 | Events API 延迟 30s-6h，不做虚假承诺 |
| 用户间社交互动/协作 | v1 做个人工具，不做社区 |
| 全自动 LLM 摘要作为核心路径 | 成本/稳定性/延迟风险高，v1 用规则+模板 |
| 多源爬取（npm/crates/PyPI） | v1 仅 GitHub API |
| Web-first 门户 | 桌面优先 |
| Org 级/Topic 级订阅 | v1 订阅对象只绑定单个 Repository |
| SurrealDB | 域模型不需要图数据库灵活性 |

## Traceability

Which phases cover which requirements. Updated during roadmap creation.

| Requirement | Phase | Status |
|-------------|-------|--------|
| FOUND-01 | Phase 1 | Pending |
| FOUND-02 | Phase 1 | Pending |
| FOUND-03 | Phase 1 | Pending |
| FOUND-04 | Phase 1 | Pending |
| TOPK-01 | Phase 2 | Pending |
| TOPK-02 | Phase 2 | Pending |
| TOPK-03 | Phase 2 | Pending |
| TOPK-04 | Phase 2 | Pending |
| TOPK-05 | Phase 2 | Pending |
| TOPK-06 | Phase 2 | Pending |
| TOPK-07 | Phase 2 | Pending |
| TOPK-08 | Phase 2 | Pending |
| SUB-01 | Phase 3 | Pending |
| SUB-02 | Phase 3 | Pending |
| SUB-03 | Phase 3 | Pending |
| SUB-04 | Phase 3 | Pending |
| SUB-05 | Phase 3 | Pending |
| SUB-06 | Phase 3 | Pending |
| SUB-07 | Phase 3 | Pending |
| SUB-08 | Phase 3 | Pending |
| RES-01 | Phase 4 | Pending |
| RES-02 | Phase 4 | Pending |
| RES-03 | Phase 4 | Pending |
| HOME-01 | Phase 3 | Pending |
| HOME-02 | Phase 3 | Pending |
| HOME-03 | Phase 5 | Pending |
| INFRA-01 | Phase 2 | Pending |
| INFRA-02 | Phase 2 | Pending |
| INFRA-03 | Phase 2 | Pending |

**Coverage:**
- v1 requirements: 29 total
- Mapped to phases: 29
- Unmapped: 0 ✅

---
*Requirements defined: 2026-03-22*
*Last updated: 2026-03-22 — Traceability mapped to 5 roadmap phases*
