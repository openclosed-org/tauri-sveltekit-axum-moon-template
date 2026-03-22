# geek taste 开发指导文档集

状态：v1.0 母稿  
面向读者：首次接触 `geek taste` 的架构师、技术负责人、前后端工程师、测试工程师、产品工程协作人员  
文档目标：在**无历史上下文**条件下，直接支持 v1 架构设计、实现拆分、开发排期、验收与后续演进。

---

## 1. 本文档集解决什么问题

本套文档集回答以下 P0 问题：

1. `geek taste` 到底是什么，不是什么。
2. v1 真实要做的能力边界是什么。
3. 关键术语、事件语义、排序语义、通知语义如何定义，避免团队成员各自理解。
4. 为什么首版推荐 `Tauri + SvelteKit + SQLite` 本地优先，以及何时引入 `Axum + Turso`。
5. 为什么不能把“任意 GitHub 仓库更新监控”设计成通用 webhook 产品。
6. 排行榜、订阅、资源雷达三条能力线如何共用同一套域模型，而不是各自为战。
7. 每个里程碑的产出、验收标准、反模式与硬约束是什么。

---

## 2. 阅读顺序（强制建议）

若你是：

- **架构负责人**：按 `00 -> 01 -> 02 -> 03 -> 04 -> 06 -> 07` 阅读。
- **前端工程师**：按 `00 -> 01 -> 04 -> 05 -> 06 -> 07` 阅读。
- **Rust / Tauri / Axum 工程师**：按 `00 -> 01 -> 03 -> 04 -> 05 -> 06 -> 07` 阅读。
- **测试 / QA**：按 `01 -> 04 -> 05 -> 06 -> 07` 阅读。

---

## 3. 文档目录

| 文件 | 作用 | 类型 |
|---|---|---|
| `00_context_and_design_master_v1.md` | v1 设计母稿，定义产品、用户、流程、信息架构 | 母稿 |
| `01_p0_domain_language_spec.md` | P0 术语、枚举、状态、事件与去歧义规范 | 规范 |
| `02_v1_scope_boundaries_assumptions_antipatterns.md` | 范围、边界、假设、非目标、反模式、决策摘要 | 规范 |
| `03_system_architecture_spec.md` | 系统架构、模块边界、部署模式、技术选型、数据流 | 架构 |
| `04_ranking_subscription_notification_spec.md` | 排行榜、订阅、资源雷达、通知、轮询与评分规则 | 业务规范 |
| `05_data_model_and_contracts.md` | 数据模型、关系约束、契约对象、示例 payload、索引规则 | 数据与接口 |
| `06_milestones_and_acceptance.md` | 里程碑、交付物、入口/出口、验收标准、DoD | 计划与验收 |
| `07_quality_security_and_operations.md` | 性能、安全、测试、监控、异常处理、发布与运行策略 | 非功能规范 |
| `08_references.md` | 外部核验资料与裁决说明 | 证据 |
| `spec/domain-vocabulary.yaml` | 机器可读的核心枚举与语义字典 | 机器规范 |
| `spec/v1-defaults.yaml` | v1 默认配置与阈值 | 机器规范 |

---

## 4. 文档使用约束

1. **母稿优先于口头描述。** 若实现与母稿冲突，以母稿和 P0 规范为准。
2. **规范优先于代码习惯。** 术语、枚举、状态机不得由各模块自行发明。
3. **边界优先于扩展欲望。** 任何新增能力先对照 `02_v1_scope_boundaries_assumptions_antipatterns.md`。
4. **验收优先于“感觉差不多”。** 任何功能完成都必须能落到 `06_milestones_and_acceptance.md` 中的场景与清单。
5. **证据纪律必须执行。** 外部事实请先看 `08_references.md`，无法核验的内容必须显式标注为 `[假设]` 或 `[推断]`。

---

## 5. 关键结论快照

### 5.1 产品定位

`geek taste` 是一个**开发者技术雷达与行动工作台**，不是泛资讯站，也不是 GitHub UI 的再包装。

### 5.2 v1 主路径

v1 只服务三类高相关任务：

1. **发现**：看语言/框架/主题下的 TopK 技术趋势。
2. **跟踪**：订阅目标仓库，接收“可用更新”级别的变化摘要。
3. **赋能**：按语言/框架发现与 code agent 生产力相关的 MCP / Skills / Agent 资源。

### 5.3 技术路线

- 首版推荐：`Tauri + SvelteKit + SQLite`，本地优先，桌面优先。
- 条件引入：当且仅当需要跨设备同步、服务端调度、Web 伴生端、集中式推送时，再引入 `Axum + Turso`。
- v1 不推荐以 `SurrealDB` 作为主数据库。

### 5.4 GitHub 约束裁决

- [核验] Tauri 官方 SvelteKit 集成要求 `static-adapter`，Tauri 不支持 server-based frontend solution。[R1][R2]
- [核验] GitHub Search API 结果存在 `1000` 条搜索结果上限与 `4000` 仓库搜索范围限制，且认证搜索限流为 `30 req/min`。[R3]
- [核验] GitHub Repository Events API 明确**不是实时接口**，事件延迟可能从 `30 秒到 6 小时`，并建议通过 `ETag` / `X-Poll-Interval` 做轮询优化。[R5]
- [核验] 仓库 webhook 的创建/管理要求仓库 owner 或 admin 权限，因此无法作为“任意公开仓库订阅”的通用监控机制。[R6]

因此，v1 订阅模型必须是：**轮询 + 差分 + 摘要 + 本地缓存**。

---

## 6. 仍然保留的开放项（P1，不阻塞开工）

1. 资源雷达是否在 v1 末期加入有限人工策展后台。
2. 是否在 v1.1 提供“高级订阅模板”（维护者模式 / 普通跟踪模式）。
3. 是否在 v1.1 支持带用户私有仓库的本地单机模式。
4. UI 设计提示词（Figma AI / 其他生成工具）将在架构母稿冻结后单独编制。

---

## 7. 修改规则

任何人要修改 P0 术语、边界、默认行为、验收标准，必须同时更新：

- `01_p0_domain_language_spec.md`
- `02_v1_scope_boundaries_assumptions_antipatterns.md`
- `04_ranking_subscription_notification_spec.md`
- `06_milestones_and_acceptance.md`
- `spec/domain-vocabulary.yaml`（若涉及枚举/状态）
- `spec/v1-defaults.yaml`（若涉及默认值/阈值）

未同步修改，视为规范变更无效。
