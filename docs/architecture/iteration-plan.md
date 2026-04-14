# 后端 L2→L3 收口迭代计划

> **目标**：将后端推到 L3（闭环平台化），前端域和集成域本轮不碰。
> **范围**：仅限后端相关 subagent（service-agent、server-agent、worker-agent、contract-agent、platform-ops-agent）的 gate 与 validator 闭环。
> **排除**：`apps/**` 治理、`gate-frontend`、integration CI 组合、E2E 集成收尾。

---

## 0. 事实基线

| 域 | 当前档位 | 说明 |
|---|---|---|
| 后端 | L2 | 结构真相源（codemap.yml）+ 局部门禁（gate-local/gate-prepush strict）已落地 |
| 前端 | L1 | 本轮不碰 |
| 集成 | 未启动 | 本轮不碰 |

### L2 → L3 差距分析

| L3 判定标准 | 当前状态 | 差距 |
|---|---|---|
| 生成物 drift 在 CI/合并前为 strict 阻塞 | warn-only | ❌ 需要切 strict |
| 合约/模型变化必须通过 generator/validator 闭环 | 部分落地 | ⚠️ 缺 resilience checks、contract drift CI |
| 变更影响有明确可执行工具链 | `scripts/route-task.ts` 已落地 | ✅ 已有基础 |
| 合并门禁为编译式裁决 | gate-local/gate-prepush 存在 | ⚠️ 需要 scoped gates + CI path-based 触发 |

---

## 1. 迭代阶段

### Phase 1：Scoped Gates 集成（当前）

**目标**：让每个 subagent 有独立可跑的 scoped gate，不再一刀切跑全量 verify。

| 步骤 | 任务 | 分配 subagent | 验收 |
|---|---|---|---|
| P1.1 | 将 `scripts/run-scoped-gates.ts` 接入 `justfiles/gates.just`，新增 `just gate-scoped <agent>` | platform-ops-agent | `just gate-scoped service-agent` 可跑 |
| P1.2 | 实现 worker resilience checks 脚本（验证 idempotency/retry/checkpoint 声明） | worker-agent | `just gate-scoped worker-agent` 通过 |
| P1.3 | 实现 server contract-check 的具体逻辑（验证 handlers 对齐 contracts） | server-agent | `just gate-scoped server-agent` 通过 |
| P1.4 | 实现 contract drift CI check（生成类型 drift 检测自动化） | contract-agent | drift 检测在 CI 中可阻塞 |

### Phase 2：Gate Strict 切换

**目标**：将 gate-local 和 gate-prepush 从 warn-only 切换到 strict 阻塞。

| 步骤 | 任务 | 分配 subagent | 验收 |
|---|---|---|---|
| P2.1 | `justfiles/gates.just` 中 `gate-local` 切 `--mode strict` | platform-ops-agent | 本地 commit 前 gate 阻塞越界修改 |
| P2.2 | `justfiles/gates.just` 中 `gate-prepush` 切 `--mode strict` | platform-ops-agent | push 前 gate 阻塞越界修改 |
| P2.3 | lefthook 验证 pre-commit / pre-push 正常阻塞 | platform-ops-agent | 故意越界修改被 lefthook 拦截 |
| P2.4 | CI 中 `quality-gate.yml` 的关键 check 切 strict | platform-ops-agent | PR 中越界修改被 CI 阻塞 |

### Phase 3：生成物 Drift 闭环

**目标**：contracts / platform model 变化必须通过 generator/validator 闭环，生成物零漂移。

| 步骤 | 任务 | 分配 subagent | 验收 |
|---|---|---|---|
| P3.1 | contract drift check 在 CI 中 strict 阻塞 | contract-agent | 修改 contracts 后不 re-gen 则 CI 失败 |
| P3.2 | platform model → catalog 生成 drift check 在 CI 中 strict 阻塞 | platform-ops-agent | 修改 model 后不 re-gen 则 CI 失败 |
| P3.3 | SDK drift check 在 CI 中 strict 阻塞 | contract-agent | 修改 contracts 后不 re-gen 则 CI 失败 |
| P3.4 | `just verify-generated` 覆盖所有生成物目录 | platform-ops-agent | 所有 generated 目录零漂移 |

### Phase 4：CI Path-Based 触发

**目标**：CI 根据 touched paths 只跑相关 gate，而不是每次 PR 跑全量。

| 步骤 | 任务 | 分配 subagent | 验收 |
|---|---|---|---|
| P4.1 | `.github/workflows/ci.yml` 增加 `scripts/route-task.ts --diff` 步骤，输出 affected subagents | platform-ops-agent | PR 中显示哪些 subagent 被影响 |
| P4.2 | CI job 按 subagent 拆分，只跑 affected 域的 scoped gates | platform-ops-agent | 只改 services/ 时不跑 app gate |
| P4.3 | summary job 按 subagent 输出状态表 | platform-ops-agent | PR summary 清晰显示各域状态 |

---

## 2. 执行顺序与依赖

```
P1 (scoped gates 集成)
  ↓
P2 (strict 切换)  ← 必须 P1 全部通过才能 strict，否则 strict 会卡住所有开发
  ↓
P3 (生成物 drift 闭环)
  ↓
P4 (CI path-based 触发)  ← 可与 P3 部分并行
```

**硬性规则**：
- P1 未完成前，P2 不允许切 strict（会导致 warn-only 门禁变 strict 后全绿但实际上有漏洞）
- P3 可与 P4 部分并行（P4.1 不依赖 P3），但 P4.2 依赖 P1（scoped gates 必须先存在）

---

## 3. Subagent 派发规则

本轮迭代只涉及以下 subagent：

| Subagent | 负责步骤 | 不参与 |
|---|---|---|
| platform-ops-agent | P1.1, P2.1, P2.2, P2.3, P2.4, P3.2, P3.4, P4.1, P4.2, P4.3 | — |
| worker-agent | P1.2 | 其他所有 |
| server-agent | P1.3 | 其他所有 |
| contract-agent | P1.4, P3.1, P3.3 | 其他所有 |
| service-agent | 本轮无专属任务（已有 gate 足够） | 全部 |
| app-shell-agent | **本轮不启用** | 全部 |

---

## 4. 完成判定（L3 验收清单）

后端达到 L3 当且仅当以下全部满足：

| # | 判定项 | 验证方式 |
|---|---|---|
| 1 | `validate-existence` 零违规 | `just gate-existence strict` |
| 2 | `validate-imports` 零违规 | `just gate-imports strict` |
| 3 | `gate-local` strict 模式下零失败 | `just gate-local` |
| 4 | `gate-prepush` strict 模式下零失败 | `just gate-prepush` |
| 5 | CI `quality-gate.yml` 零失败 | 创建包含后端修改的 PR 验证 |
| 6 | Contract drift 在 CI 中 strict 阻塞 | 修改 contracts 后不 re-gen，CI 失败 |
| 7 | Platform model drift 在 CI 中 strict 阻塞 | 修改 model 后不 re-gen，CI 失败 |
| 8 | SDK drift 在 CI 中 strict 阻塞 | 修改 contracts 后不 re-gen，CI 失败 |
| 9 | `just verify-generated` 覆盖所有生成物 | 手动改一个 generated 文件，`just verify-generated` 检测出 |
| 10 | CI path-based 触发生效 | 只改 services/，CI 只跑 service gate |

---

## 5.  excluded 范围（本轮明确不碰）

| 域 | 原因 |
|---|---|
| `apps/**` 治理 | 前端 L1，本轮不碰 |
| `gate-frontend` | 前端专属门禁，本轮不碰 |
| `apps/web` 目录重构 | 前端域，本轮不碰 |
| Integration CI 组合 | 前端未稳，integration 不碰 |
| E2E 集成收尾 | 依赖前端稳定，本轮不碰 |
| `packages/ui/**` | 前端域，本轮不碰 |

---

## 6. Agent 阅读协议

后续任何 agent 开始工作时：

1. 读 `AGENTS.md`（总控协议）
2. 读 `agent/codemap.yml`（模块约束）
3. 读本文件（迭代计划）
4. 读 `docs/architecture/maturity-levels.md`（档位定义）
5. 根据自身负责的步骤执行

禁止：
- 读取或实施 excluded 范围内的任务
- 同时推进多个 Phase（严格按 P1→P2→P3→P4 顺序）
- 在 P1 完成前切 P2 的 strict
