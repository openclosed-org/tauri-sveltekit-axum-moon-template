# Agent 上下文流 P0 重构计划

> **生成时间**: 2026-04-12
> **核心目标**: 让 Agent 每次启动时的上下文流**准确、无歧义、不过期、不失真**
> **原则**: 只分析不修改，给出完整可执行方案

---

## 1. 当前问题诊断

### 1.1 什么是 Agent 上下文流？

Agent 上下文流 = Agent 每次任务启动时必须读取的最小文档集合，它定义了：
- 项目是什么（README.md）
- 如何协作（AGENTS.md）
- 目录规则（repo-layout.md）
- 模块约束（codemap.yml）

**当前问题**：这 4 个核心文档中，有 3 个包含失效链接、过时描述、自相矛盾的信息，会直接误导 Agent。

### 1.2 当前污染源清单

| 文件 | 污染类型 | 影响 |
|------|---------|------|
| `README.md` | 3 个失效链接 + 完成度表失真 + 描述不存在的目录 | 🔴 高 — Agent 第一个读取的文件 |
| `AGENTS.md` | 引用不存在的 `docs/CURRENT-STATE.md` 和 `docs/architecture-gap-priority-plan.md` | 🔴 高 — 要求 Agent 必读但文件不存在 |
| `docs/ARCHITECTURE.md` | 描述 50+ 模块的目标态，无"这是目标"警告 | 🟡 中 — 容易让 Agent 误判当前状态 |
| `docs/refactoring/` 整个目录 | 重构历史文档，项目已进入增量开发阶段 | 🟡 中 — 污染 Agent 上下文 |
| `.refactoring-state.yaml` | phase 全 completed 但 progress 70% 自相矛盾 | 🟡 中 — 失真 |
| 所有一级目录的 README.md | `servers/README.md`、`services/README.md`、`packages/README.md` | 🟡 中 — 与 repo-layout.md 重复且失真 |

---

## 2. 目标架构：Agent 上下文流设计

### 2.1 四个必读文档（Agent 每次启动必读）

```
Agent 启动 → 按顺序读取:

1. README.md                    # 项目一句话介绍 + 索引入口
2. AGENTS.md                    # AI 协作协议 + 工程规则
3. docs/architecture/repo-layout.md  # 目录布局规则（真理源）
4. agent/codemap.yml            # 模块约束与依赖规则（真理源）

必要时扩展读取:
5. agent/ 所有文档              # 模板、checklist、约束、边界
6. docs/ 所有文档               # ADR、架构图、运维指南、合约
```

### 2.2 文档职责分层

| 文档 | 职责 | 维护原则 |
|------|------|---------|
| `README.md` | **索引入口** — 一句话介绍 + 4 个核心文档链接 + 开发命令 | 极简，不描述具体架构细节 |
| `AGENTS.md` | **协作协议** — AI 行为规则、工具使用、禁止目录、工作流 | 通用工程规则，不随业务变化 |
| `repo-layout.md` | **目录规则** — 每个顶级目录的职责、必须、禁止、验证 | 目标态描述，缺失子目录用 `.gitkeep` 占位 |
| `codemap.yml` | **模块约束** — 每个模块的路径、依赖、禁止、文件要求 | 目标态描述，未实现模块先定义后占位 |

### 2.3 文档淘汰策略

| 淘汰 | 原因 |
|------|------|
| `docs/ARCHITECTURE.md` | 内容已分发到 `repo-layout.md` + `codemap.yml`，完成分发后删除 |
| 所有一级目录 `README.md` | `servers/README.md`、`services/README.md`、`packages/README.md` 等 — 内容已被 `repo-layout.md` 覆盖 |
| `docs/CURRENT-STATE.md` | 不需要 — repo-layout.md 和 codemap.yml 是目标态，Agent 以代码为准 |
| `docs/architecture-gap-priority-plan.md` | 不需要 — 重构已完成，进入增量开发 |
| `.refactoring-state.yaml` | 不需要 — 重构已完成 |
| `docs/refactoring/` 整个目录 | 不需要 — 重构历史不应污染 Agent 上下文 |

---

## 3. 执行计划

### Phase 1: 清理污染（删除过时文档）

**目标**: 删除所有重构历史文档和失效 tracker

| # | 操作 | 文件 | 原因 |
|---|------|------|------|
| 1.1 | 🗑️ 删除 | `.refactoring-state.yaml` | 重构完成，phase tracker 不再有意义 |
| 1.2 | 🗑️ 删除 | `docs/refactoring/audit-report-2026-04-12.md` | 上一轮审计产物，问题应已修复 |
| 1.3 | 🗑️ 删除 | `docs/refactoring/p0-analysis-2026-04-12.md` | 上一轮分析产物，计划应已执行 |
| 1.4 | 🗑️ 删除 | `docs/refactoring/` 目录（删空后移除） | 空目录，不应存在 |
| 1.5 | 🗑️ 删除 | `docs/report/` 目录（如存在且无内容） | 检查后决定 |

**风险**: 🟢 零风险 — 纯历史文件，不影响构建和运行

---

### Phase 2: 重写四个核心文档

#### 2.1 重写 `README.md`

**当前问题**: 3 个失效链接 + 失真完成度表 + 描述不存在的目录

**新内容**:

```markdown
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

1. 平台模型优先
2. 契约先于实现
3. Services 是库，不是进程
4. Workers 是一等公民
5. Vendor 只能进 adapters
6. 生成物禁止手改
7. 拓扑切换靠 topology model，不靠重构
```

**变更**: 删除快速导航表、完成度表、重要提醒。改为极简索引。

---

#### 2.2 修正 `AGENTS.md`

**当前问题**: §1.1 引用了不存在的 `docs/CURRENT-STATE.md` 和 `docs/architecture-gap-priority-plan.md`

**修正方案**: 重写 §0.1 和 §1.1 为新的四文档索引：

```markdown
## 0. 任务启动清单

### 0.1 必读文档（每次任务按顺序执行）

```
1. 读取 AGENTS.md（本文）→ 了解协作协议
2. 读取 docs/architecture/repo-layout.md → 了解目录布局规则
3. 读取 agent/codemap.yml → 了解模块约束
4. 必要时读取 agent/ 全部文档和 docs/ 全部文档
```

### 0.2 文档优先级

| 优先级 | 文档 | 用途 |
|-------|------|------|
| **P0** | `docs/architecture/repo-layout.md` | 目录布局规则，每个目录的职责/必须/禁止/验证 |
| **P0** | `agent/codemap.yml` | 模块级约束：路径、依赖、禁止、文件要求 |
| **P1** | `AGENTS.md`（本文） | AI 协作协议、工具规则、禁止目录 |
| **P2** | `agent/constraints/` | 机器可读的依赖/模式/契约约束 |
| **P2** | `docs/adr/` | 架构决策记录（001-008） |
| **P2** | `docs/architecture/` | C4 架构图与 sync 流程 |
| **P2** | `docs/contracts/` | HTTP/Event/RPC 协议文档 |
| **P2** | `docs/operations/` | 运维指南（local-dev/single-vps/k3s/gitops/secrets） |
```

**硬性规则**:
- 遇到文档与代码冲突，**以代码为准**
- 不得仅凭 codemap.yml 推断"文件应该存在"（未实现模块可能有 .gitkeep）
- `repo-layout.md` 和 `codemap.yml` 描述的是目标态，缺失部分用 `.gitkeep` 占位

**删除**: 旧的 §1.1 整个表格（引用不存在文件）。

---

#### 2.3 修正 `docs/architecture/repo-layout.md`

**当前状态**: 基本准确，描述的是目录规则和目标态。

**需要调整**:
1. 确认 §3 中描述的每个顶级目录都与实际一致
2. 对于当前缺失但目标态应有的子目录，在规则后加一行：
   > 注：部分子目录当前为空（含 `.gitkeep`），待后续增量开发时填充。
3. 删除或更新任何引用 `ARCHITECTURE.md` 的地方
4. 确认 §6 新增模块模板与实际模板目录对应（`agent/templates/` 当前只有 `bff-endpoint/` 和 `module/`）

**风险**: 🟢 低 — 纯文档修正

---

#### 2.4 修正 `agent/codemap.yml`

**当前问题**: `modules` 部分定义了多个实际不存在的模块，路径也不匹配实际命名。

**需要调整的内容**:

| 当前定义 | 实际状态 | 操作 |
|---------|---------|------|
| `services/user` path: `services/user` | 实际: `services/user-service` | ✅ 修正 path |
| `services/tenant` path: `services/tenant` | 实际: `services/tenant-service` | ✅ 修正 path |
| `services/settings` path: `services/settings` | 实际: `services/settings-service` | ✅ 修正 path |
| `services/counter` path: `services/counter` | 实际: `services/counter-service` | ✅ 修正 path |
| `services/admin` path: `services/admin` | 实际: `services/admin-service` | ✅ 修正 path |
| `services/indexing` path: `services/indexing` | ❌ 不存在 | 🟢 **保留定义** — 这是目标态，加 `.gitkeep` 占位 |
| `servers/web-bff` path: `servers/web-bff` | 实际: `servers/bff/web-bff` | ✅ 修正 path |
| `servers/admin-bff` path: `servers/admin-bff` | 实际: `servers/bff/admin-bff` | ✅ 修正 path |
| `servers/edge-gateway` path: `servers/edge-gateway` | 实际: `servers/gateway`（占位） | ✅ 修正 path |
| `servers/internal-rpc` | ❌ 不存在 | 🟢 **保留定义** — 目标态，加 `.gitkeep` 占位 |
| `workers/workflow-runner` | ❌ 不存在 | 🟢 **保留定义** — 目标态，加 `.gitkeep` 占位 |
| `apps/mobile` | ❌ 不存在 | 🟢 **保留定义** — 目标态，加 `.gitkeep` 占位 |

**核心原则**: 
- **已存在的模块**: 修正 path 为实际路径
- **未实现但目标态应有的模块**: 保留定义，不删除。对应目录加 `.gitkeep` 占位
- **codemap.yml 是目标态真理源**，不应因当前未实现而删减

**新增**: 加一个 `notes` 字段说明：
```yaml
notes: >
  本文档描述的是目标态模块结构。
  部分模块尚未实现，对应目录以 .gitkeep 占位。
  新增模块时，必须先更新本文档，再创建目录和代码。
```

---

### Phase 3: 分发 ARCHITECTURE.md 内容后删除

#### 3.1 内容分发检查

`docs/ARCHITECTURE.md` 当前 1330 行，包含：
- §1 目录树模板 → 已覆盖在 `repo-layout.md` §2
- §2 全局硬性约束 → 部分内容在 `repo-layout.md` §4 + `codemap.yml` rules
- §3 各顶级目录硬规则 → 已覆盖在 `repo-layout.md` §3
- §4 关键子目录硬性规则 → 部分在 `repo-layout.md` §3 子节
- §5 命名规则 → 已覆盖在 `repo-layout.md` §5
- §6 新增模块最小模板 → 已覆盖在 `repo-layout.md` §6
- §7 推荐统一命令 → 已覆盖在 `repo-layout.md` §7
- §8 七条规则 → 已覆盖在 README.md 核心原则
- §9 校验命令 → 应在 `codemap.yml` validation 部分

**结论**: `ARCHITECTURE.md` 的内容已充分分发到 `repo-layout.md` + `codemap.yml` + `README.md`。

#### 3.2 删除步骤

```
3.1 确认 repo-layout.md 和 codemap.yml 已包含 ARCHITECTURE.md 的所有规则
3.2 确认 README.md 核心原则包含 §8 七条规则
3.3 确认 codemap.yml validation 包含 §9 校验命令
3.4 删除 docs/ARCHITECTURE.md
```

**风险**: 🟡 中 — 删除前必须确认内容完全分发

---

### Phase 4: 删除所有一级目录 README.md

| 文件 | 内容是否被 repo-layout.md 覆盖 | 操作 |
|------|------|------|
| `servers/README.md` | ✅ 是 | 🗑️ 删除 |
| `services/README.md` | ✅ 是 | 🗑️ 删除 |
| `packages/README.md` | ✅ 是 | 🗑️ 删除 |
| `packages/LAYERING.md` | ⚠️ 有额外分层说明 | 🟢 保留或移入 docs/ |
| `infra/README.md` | ✅ 是 | 🗑️ 删除 |
| `verification/README.md` | ✅ 是 | 🗑️ 删除 |
| `agent/README.md` | ⚠️ 可能有 agent 特定说明 | 🟢 检查后决定 |

**注意**: `platform/README.md` 如果存在也应删除（被 repo-layout.md §3.3 覆盖）。

---

### Phase 5: 补充 `.gitkeep` 占位

根据 `codemap.yml` 中保留但尚未实现的模块，在对应目录加 `.gitkeep`：

| 目标模块 | 应创建的占位目录 | .gitkeep 位置 |
|---------|---------------|--------------|
| `services/indexing` | `services/indexing-service/src/{domain,application,policies,ports,events,contracts}/` | 每个子目录 |
| `servers/internal-rpc` | `servers/internal-rpc/src/` | `src/` |
| `servers/edge-gateway` | `servers/gateway/src/{authn,authz,rate_limit,routing,observability}/` | 每个子目录 |
| `workers/workflow-runner` | `workers/workflow-runner/src/` | `src/` |
| `apps/mobile` | `apps/mobile/src/{lib/api,lib/sync}/` | 每个子目录 |
| `packages/web3` | `packages/web3/` | 根目录 |
| `packages/wasm` | `packages/wasm/` | 根目录 |
| `packages/data` | `packages/data/` | 根目录 |
| `packages/messaging` | `packages/messaging/` | 根目录 |
| `packages/cache` | `packages/cache/` | 根目录 |
| `packages/storage` | `packages/storage/` | 根目录 |
| `packages/observability` | `packages/observability/` | 根目录 |
| `packages/security` | `packages/security/` | 根目录 |
| `packages/authn` | `packages/authn/` | 根目录 |
| `packages/authz` | `packages/authz/` | 根目录 |
| `packages/devx` | `packages/devx/` | 根目录 |

**原则**: 
- 只创建必要的目录骨架 + `.gitkeep`
- 不创建 `Cargo.toml` 或 `package.json`（避免污染 workspace）
- 每个 `.gitkeep` 所在目录加一行注释说明用途

---

## 4. 执行顺序与依赖

```
Phase 1: 清理污染（删除历史文档）
  ├── 无依赖，可立即执行
  └── 风险: 🟢 零

Phase 2: 重写四个核心文档
  ├── 2.1 README.md — 无依赖
  ├── 2.2 AGENTS.md — 无依赖
  ├── 2.3 repo-layout.md — 无依赖
  └── 2.4 codemap.yml — 无依赖
  └── 风险: 🟡 中（codemap.yml 影响 Agent 约束检查）

Phase 3: 删除 ARCHITECTURE.md
  ├── 依赖: Phase 2 完成（repo-layout.md + codemap.yml 已完善）
  └── 风险: 🟡 中（删除前必须确认内容完全分发）

Phase 4: 删除一级目录 README.md
  ├── 依赖: Phase 2 完成（repo-layout.md 已完善）
  └── 风险: 🟢 低

Phase 5: 补充 .gitkeep 占位
  ├── 依赖: Phase 2.4 完成（codemap.yml 目标态模块已确认）
  └── 风险: 🟢 低（只创建空目录 + .gitkeep）
```

---

## 5. 验证标准

完成后，Agent 启动时应满足：

| 验证项 | 预期结果 |
|--------|---------|
| 读取 `README.md` 中的链接 | 所有链接指向存在的文件 |
| 读取 `AGENTS.md` §0.1 的文档列表 | 所有文档存在 |
| 读取 `repo-layout.md` | 描述的目录规则与实际目录一致 |
| 读取 `codemap.yml` modules | path 字段指向实际存在或 `.gitkeep` 占位的路径 |
| 检查 `docs/refactoring/` | 不存在（已清理） |
| 检查 `.refactoring-state.yaml` | 不存在（已清理） |
| 检查 `docs/ARCHITECTURE.md` | 不存在（已分发后删除） |
| 检查一级目录 README.md | 不存在（已被 repo-layout.md 覆盖） |
| 检查 `agent/constraints/` 缺失文件 | 要么补齐，要么从 `repo-layout.md` 模板描述中移除 |

---

## 6. 风险矩阵

| 操作 | 风险 | 回滚难度 | 缓解措施 |
|------|------|---------|---------|
| Phase 1 删除历史文档 | 🟢 无 | 无 | git 可恢复 |
| Phase 2.1 重写 README | 🟢 低 | 极低 | 纯文档 |
| Phase 2.2 修正 AGENTS.md | 🟡 中 | 低 | 影响 Agent 行为 |
| Phase 2.3 修正 repo-layout.md | 🟢 低 | 低 | 纯文档 |
| Phase 2.4 修正 codemap.yml | 🟡 中 | 中 | Agent 约束检查依赖此文件 |
| Phase 3 删除 ARCHITECTURE.md | 🟡 中 | 中 | 必须先确认内容分发完成 |
| Phase 4 删除目录 README | 🟢 低 | 低 | 纯文档 |
| Phase 5 创建 .gitkeep | 🟢 低 | 极低 | 空目录不影响构建 |

---

## 7. 后续工作（本计划范围外）

完成 Agent 上下文流清理后，后续可执行：

1. **P0-1: 删除 `servers/api/`** — 见之前的分析报告，需要先迁移能力到 web-bff
2. **补齐 `agent/constraints/` 缺失文件** — telemetry-policy.yaml, authz-policy.yaml, topology-policy.yaml
3. **补齐 `agent/templates/` 缺失模板** — service/, worker/, contract/, platform-model/
4. **补齐 `agent/prompts/` 缺失文件** — add-service.md, add-worker.md, add-bff.md, add-resource.md, add-contract.md
5. **更新 CI 配置** — 确保 CI 只引用存在的文档和命令

---

## 8. 总结

**核心认知转变**:

1. `repo-layout.md` 和 `codemap.yml` 是**目标态**文档，不是当前态文档
2. 缺失的模块不应该从这两个文档中删除，而应该在代码库中用 `.gitkeep` 占位
3. Agent 不需要知道"哪些还没做"，只需要知道"最终应该是什么"和"当前实际有什么"
4. 所有重构历史文档都应该删除，进入增量开发阶段后不再保留

**本次 P0 的范围**: Phase 1-5，清理污染 + 重写四文档 + 删除 ARCHITECTURE.md + 删除目录 README + 补充 .gitkeep。

**预期结果**: Agent 每次启动时读取的上下文流 100% 准确，无失效链接，无自相矛盾，无过时描述，无重构历史污染。
