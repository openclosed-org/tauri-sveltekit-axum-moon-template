# AGENTS.md — 仓库级 AI 协作协议

> **适用场景**：100% Agent 开发、维护、迭代的 Tauri 2 + SvelteKit + Axum 跨端项目。
> **核心原则**：先读再改、最小改动、可验证结果。禁止猜测、禁止表面通过。

---

## 0. 任务启动清单

每次收到任务后，按顺序执行：

### 0.1 加载索引

```
读取 agent/directory_categories.json → 根据任务类型确定优先搜索分类
```

| 任务类型 | 优先搜索分类 |
|----------|-------------|
| 前端 UI/交互 | `frontend` + `shared` + `contracts` |
| 后端/API/服务 | `backend` + `shared` + `contracts` |
| 全栈/跨层 | `frontend` + `backend` + `shared` + `contracts` |
| 基础设施/运维 | `infra` + `agent` |
| 测试相关 | 对应领域分类 + `tests` |
| Agent 约束/模板 | `agent` |

**这是搜索优先级提示，不是硬过滤。** 任何任务都可能需要跨越分类边界。

### 0.2 必读根配置文件

涉及代码改动前，先确认理解了项目的基础设施：

| 文件 | 何时读取 |
|------|---------|
| `Cargo.toml`（根） | 修改 Rust 代码、新增依赖、改 workspace 成员 |
| `moon.yml` | 运行/调试/新增构建任务 |
| `Justfile` + `justfiles/*.just` | 运行/新增命令 |
| `.mise.toml` | 工具版本问题 |
| `rustfmt.toml` / `clippy.toml` / `biome.json` | 格式/风格问题 |
| `bun-workspace.yaml` / `package.json` | 前端依赖问题 |
| `.gitignore` | 新增文件是否需要忽略 |

### 0.3 明确目标架构与当前状态

**项目定位**：模块化单体（Monolith），架构设计先行，后续无痛微服务化。详见 [docs/GOAL.md](docs/GOAL.md)。

**目标依赖方向（不可违反）**：

```
packages/contracts/  ←  所有共享类型的单一真理源（HTTP/Event/DTO）
        ↑
packages/features/   ←  定义 trait + 类型，不得包含实现，不得依赖 services
        ↑
services/            ←  领域服务（Clean Architecture 四层：domain/ports/application/infrastructure）
        ↑
packages/core/       ←  系统级抽象（TenantId/Error/Config/Clock）+ 基础设施端口
        ↑
packages/adapters/   ←  外部世界翻译层（Turso/SurrealDB/Telemetry/Auth），不得承载业务逻辑
        ↑
servers/             ←  组合层（Axum 路由注册 + 中间件），不得包含业务逻辑
        ↑
apps/                ←  纯展示层（SvelteKit 渲染 + 用户交互）
```

**当前服务状态（Agent 必须知道代码实际在哪里）**：

| 服务 | 状态 | 代码位置 |
|------|------|---------|
| counter | ✅ 完整 | `services/counter-service/`（黄金示例） |
| user | ✅ 完整 | `services/user-service/`（缺 HTTP 路由） |
| tenant | ⚠️ 待迁移 | 业务在 `packages/core/usecases/tenant_service.rs`，目录是 stub |
| agent | ⚠️ 待迁移 | 业务在 `packages/core/usecases/agent_service.rs`，目录是 stub |
| admin | ⚠️ 待迁移 | 业务在 `packages/core/usecases/admin_service.rs`，目录不存在 |
| chat | ❌ 待实现 | 目录是 stub，Tauri 有命令但服务端无实现 |
| event-bus | ✅ 部分 | `services/event-bus/`（内存实现 + Outbox） |

**关于 `servers/` 与 BFF 的关系**：
`servers/api/` 是单体 Axum 服务器，聚合所有路由。`servers/bff/web-bff/` 是 Web 端 BFF（已存在但路由不完整）。
**BFF 是后端组合层，不是前端的一部分。**

**生成新代码时的规则**：
- 新增**业务模块** → 在 `services/<domain>/` 下创建（参考 counter-service 黄金示例）
- 新增**共享 trait/类型** → `packages/features/<domain>/` 或 `packages/core/`
- 新增**基础设施实现** → `packages/adapters/`
- 新增**HTTP 端点** → `servers/api/src/routes/` 和 `servers/bff/web-bff/src/handlers/`
- 新增**前端页面** → `apps/web/src/routes/`
- ❌ **禁止**在 `packages/core/usecases/` 中新增业务逻辑（历史遗留，正在清空）

### 0.4 确定验证方式

在动手改代码之前，确认：
- 改完后运行什么命令验证？（查 `Justfile` 和 `moon.yml`）
- 相关测试在哪里？（查 `directory_categories.json` 的 `tests` 分类）
- 影响的范围有多大？（`git diff --stat` + `cargo check -p <package>`）

---

## 1. 全局硬约束

1. **中文沟通**；代码、命令、配置键、日志、协议字段保持原文。
2. **先读再改**：未审查现状就重写 = 制造回归风险。
3. **先证据后判断**：遇到报错，先获取完整日志和复现步骤，再分析。
4. **先搜索后猜测**：陌生 API / 新框架 → 查文档、issues、release notes，不要反复试过期写法。
5. **先小后大**：优先最小闭环、局部改动、可回滚方案。
6. **修改前先解释**：当前设计可能在保护什么？不要随意改变可观察行为、接口形状、错误语义、默认值。
7. **未执行 ≠ 已执行**：未实际运行的验证步骤，不得声称通过。
8. **不确定 = 明确说明**：标注不确定点、影响范围、后续验证方式。
9. **禁止绕过问题**：注释/删除/跳过关键逻辑、吞掉错误、伪造成功状态，等同于制造 bug。

---

## 2. 工程偏置

### 2.1 架构

- **adapter 薄，core 稳**：Tauri command / Axum handler 只做适配与协议翻译
- **SvelteKit 不承载领域真相**：UI 组合 + 瞬时状态，业务逻辑在 `services/`
- **跨 Rust/TypeScript 边界优先 typed contracts**：由 `packages/contracts/` 生成
- **服务之间不得直接依赖**：必须通过 `contracts/events` 通信
- **宁可小范围重复，不要过早抽象**
- **同一业务概念在不同层的类型必须对齐**，不得出现字段级差异

### 2.2 技术决策优先级

满足需求 → 正确性 → 回归风险 → 复用现有模式 → 可测试性 → 交付速度 → 扩展性

### 2.3 禁止

- 非必要不引入新依赖、新增抽象层、修改目录结构、大规模重构
- 不顺手修复无关问题（除非阻塞当前任务）
- 不把猜测包装成结论，把未验证包装成完成

---

## 3. 工具使用原则

### 3.1 优先使用

| 场景 | 工具 |
|------|------|
| 代码搜索 | `grep_search`（ripgrep）、`agent`（open-ended） |
| 文件查找 | `glob` |
| 差异分析 | `git diff --stat`、`git log -- <path>` |
| 结构化处理 | `jq`、`yq` |

### 3.2 规则

- 工具输出必须结合仓库上下文解释，不能机械照搬
- 能用 tool 做的事不要手写（搜索用 `grep_search` 不用 shell `grep`）
- 多个独立搜索可以并行执行

---

## 4. 标准工作流

```
1. 明确目标      →  目标、输入、输出、约束、验收标准、风险
2. 审查现状      →  读相关代码/配置/文档/测试，理解现有实现
3. 制定方案      →  最小闭环，多方案时说明取舍
4. 实施改动      →  局部化，命名/风格/结构与现有约定一致
5. 验证结果      →  运行直接相关的测试/构建/检查
6. 输出结果      →  改了什么、为什么这样改、验证程度、剩余风险
```

---

## 5. 风险升级

遇到以下情况必须显式提示风险，不得闷头推进：

1. 需求与现有架构明显冲突
2. 改动影响多个核心模块或公共契约
3. 需要新增关键依赖或改动关键链路
4. 测试缺失导致无法可靠验证
5. 技术债已使继续叠加改动风险过高

---

## 6. 禁止读取的目录

以下目录是构建产物或外部缓存，**永远不要读取或搜索其中的内容**：

| 目录 | 原因 |
|------|------|
| `node_modules/` | 第三方依赖，不修改 |
| `target/` | Rust 构建产物，随时可重新生成 |
| `.moon/cache/` | moon 缓存 |
| `.cocoindex_code/` | 索引缓存 |
| `.jj/` | Jujutsu VCS 内部数据 |

---

## 7. 执行底线

1. 不回避问题，不制造表面通过
2. 不盲猜，不重复试错同一类过期方案
3. 不为了"架构好看"牺牲交付效率
4. 不为了短期通过破坏长期可维护性
5. 始终优先：真实证据、局部改动、可验证结果
