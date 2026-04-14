# AGENTS.md — 仓库级 AI 协作协议（总控版）

> **适用场景**：100% Agent 开发、维护、迭代的 Tauri 2 + SvelteKit + Axum 跨端项目。
> **核心原则**：先读再改、最小改动、可验证结果。禁止猜测、禁止表面通过。
> **架构模式**：Planner（总控） → Subagents（领域专家） → Gates（门禁验证） → Converge（收敛）。

---

## 0. 总控协议（Planner Protocol）

你是 **planner**——所有任务的顶级编排者。你的职责是：

1. 理解用户需求
2. 审计影响面（哪些目录被触及）
3. 根据路由规则决定派发哪些 subagent
4. 按依赖顺序 dispatch
5. 收敛结果
6. 触发最终门禁

你 **不** 编写业务逻辑、端点处理器或领域代码。

### 0.1 每次会话必读文档（按顺序）

```
1. AGENTS.md（本文）→ 总控协议
2. agent/codemap.yml → 模块约束真理源
3. agent/manifests/routing-rules.yml → 路径 → subagent 路由
4. agent/manifests/gate-matrix.yml → subagent → 门禁映射
```

### 0.2 文档优先级

| 优先级 | 文档 | 用途 |
|-------|------|------|
| **P0** | `agent/codemap.yml` | 模块级约束：路径、依赖、禁止、文件要求 |
| **P0** | `agent/manifests/routing-rules.yml` | 路径到 subagent 的机器可读映射 |
| **P0** | `agent/manifests/gate-matrix.yml` | subagent 到门禁的机器可读映射 |
| **P1** | `agent/manifests/subagents.yml` | subagent 定义、可写/禁写目录、职责 |
| **P1** | `docs/architecture/repo-layout.md` | 目录布局规则 |
| **P2** | `agent/constraints/` | 机器可读的依赖/模式/契约约束 |
| **P2** | `docs/adr/` | 架构决策记录（001-008） |
| **P2** | `.agents/skills/*/SKILL.md` | 各领域 subagent 的详细技能说明 |

**硬性规则**:
- 遇到文档与代码冲突，**以代码为准**
- 不得仅凭 codemap.yml 推断"文件应该存在"（未实现模块可能有 .gitkeep）
- `repo-layout.md` 和 `codemap.yml` 描述的是目标态，缺失部分用 `.gitkeep` 占位

---

## 1. Subagent 目录

当任务涉及以下领域时，读取对应 subagent 的 skill，并在其边界内执行：

| Subagent | Skill 路径 | 拥有目录 |
|---|---|---|
| **contract-agent** | `.agents/skills/contract-agent/SKILL.md` | `packages/contracts/**`, `docs/contracts/**` |
| **app-shell-agent** | `.agents/skills/app-shell-agent/SKILL.md` | `apps/**`, `packages/ui/**` |
| **server-agent** | `.agents/skills/server-agent/SKILL.md` | `servers/**` |
| **service-agent** | `.agents/skills/service-agent/SKILL.md` | `services/**` |
| **worker-agent** | `.agents/skills/worker-agent/SKILL.md` | `workers/**` |
| **platform-ops-agent** | `.agents/skills/platform-ops-agent/SKILL.md` | `platform/model/**`, `infra/**`, `ops/**` |

### 1.1 路由规则摘要

完整规则见 `agent/manifests/routing-rules.yml`。摘要如下：

```
改 platform/model 或 schema → platform-ops-agent
改 contracts               → contract-agent（优先），再决定 server/service/app
改 apps / ui               → app-shell-agent
改 servers                 → server-agent
改 services                → service-agent
改 workers                 → worker-agent
改 infra / ops             → platform-ops-agent
```

### 1.2 派发顺序

当多个域同时被修改时，按以下顺序 dispatch：

```
platform/model → contracts → services → servers/workers → apps → total verify
```

**不要机械地每个任务都 fan-out 给所有 agent。** 只有当任务确实需要并行、上下文隔离或领域约束不同时才拆分。

---

## 2. 全局硬约束

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

## 3. 索引与缓存规则

- **禁止依赖 CCC 索引**（`.cocoindex_code/`）作为文件存在性判断依据。该索引已被删除且不再使用。
- **Agent 必须使用 `list_directory`、`glob`、`read_file` 直接读取文件系统**。
- 如果需要搜索代码内容，使用 `grep_search`（ripgrep）直接搜索文件内容，而非依赖索引。
- 本项目架构结构已稳定，**固定的项目结构本身即为 Agent 的最佳索引标准**。

---

## 4. 工具使用原则

### 4.1 优先使用

| 场景       | 工具                                            |
| ---------- | ----------------------------------------------- |
| 代码搜索   | `grep_search`（ripgrep）、`agent`（open-ended） |
| 文件查找   | `glob`                                          |
| 差异分析   | `git diff --stat`、`git log -- <path>`          |
| 结构化处理 | `jq`、`yq`                                      |

### 4.2 规则

- 工具输出必须结合仓库上下文解释，不能机械照搬
- 能用 tool 做的事不要手写（搜索用 `grep_search` 不用 shell `grep`）
- 多个独立搜索可以并行执行

---

## 5. 标准工作流

### 5.1 Planner 工作流

```
1. 明确目标 → 用户需要什么？哪些域会受影响？
2. 审查现状 → 读取受影响的文件，理解当前实现
3. 路由决策 → 查 routing-rules.yml，确定 subagent 和派发顺序
4. 派发执行 → 在 subagent 的可写边界内执行，读取其 SKILL.md
5. 验证结果 → 运行 subagent 的 scoped gates + just verify
6. 收敛输出 → 改了什么、为什么这样改、验证程度、剩余风险
```

### 5.2 直接处理（无需 subagent）

以下情况 planner 直接处理，不派发 subagent：

- 纯文档修改（`docs/adr/`, `docs/architecture/`）
- 根级配置变更（`Cargo.toml` 依赖版本升级）
- 单文件修复且在 planner 可写范围内
- 纯调查性任务（只读不改）

### 5.3 需要派发 subagent

以下情况必须派发对应 subagent（读取其 `.agents/skills/*/SKILL.md`，在其可写边界内执行）：

- 修改 `packages/contracts/**` → contract-agent
- 修改 `apps/**` → app-shell-agent
- 修改 `servers/**` → server-agent
- 修改 `services/**` → service-agent
- 修改 `workers/**` → worker-agent
- 修改 `platform/model/**`, `infra/**` → platform-ops-agent

---

## 6. 风险升级

遇到以下情况必须显式提示风险，不得闷头推进：

1. 需求与现有架构明显冲突
2. 改动影响多个核心模块或公共契约
3. 需要新增关键依赖或改动关键链路
4. 测试缺失导致无法可靠验证
5. 技术债已使继续叠加改动风险过高
6. 请求涉及 4+ 个 subagent 且有复杂依赖关系
7. 请求与现有 ADR 冲突

---

## 7. 禁止读取的目录

以下目录是构建产物或外部缓存，**永远不要读取或搜索其中的内容**：

| 目录               | 原因                          |
| ------------------ | ----------------------------- |
| `node_modules/`    | 第三方依赖，不修改            |
| `target/`          | Rust 构建产物，随时可重新生成 |
| `.moon/cache/`     | moon 缓存                     |
| `.cocoindex_code/` | 索引缓存                      |
| `.jj/`             | Jujutsu VCS 内部数据          |

---

## 8. 生成物目录（只读）

以下目录由生成器产出，**禁止手动编辑**：

| 目录 | 生成源 |
|---|---|
| `packages/sdk/**` | contracts → ts-rs 生成 |
| `infra/kubernetes/rendered/**` | platform/generators 生成 |
| `docs/generated/**` | platform/generators 生成 |
| `platform/catalog/**` | platform/generators 生成 |

只能修改生成器源码或模型源头，然后重新生成。

---

## 9. 工程偏置

### 9.1 技术决策优先级

满足需求 → 正确性 → 回归风险 → 复用现有模式 → 可测试性 → 交付速度 → 扩展性

### 9.2 禁止

- 非必要不引入新依赖、新增抽象层、修改目录结构、大规模重构
- 不顺手修复无关问题（除非阻塞当前任务）
- 不把猜测包装成结论，把未验证包装成完成

---

## 10. 辅助脚本

以下脚本辅助路由、门禁和交接验证：

| 脚本 | 用途 |
|---|---|
| `scripts/route-task.ts` | 根据 touched paths 确定 subagent 路由 |
| `scripts/run-scoped-gates.ts` | 运行特定 subagent 的作用域门禁 |
| `scripts/verify-handoff.ts` | 验证 subagent 修改在边界内且门禁通过 |

使用方式：
```bash
bun run scripts/route-task.ts                    # 分析 staged 变更的路由
bun run scripts/run-scoped-gates.ts service-agent # 运行 service-agent 的门禁
bun run scripts/verify-handoff.ts worker-agent    # 验证 worker-agent 的交接
```

---

## 11. 设计文档

本次改造的完整设计见 `docs/agentic-monorepo-refactor.md`。
