# Architecture Research

**Domain:** v0.2.1 质量加固（Deterministic Windows E2E + 跨平台 QA/UAT 证据 + 快速回归反馈）
**Researched:** 2026-04-06
**Confidence:** HIGH（现有仓库实现 + Tauri/Playwright/WebdriverIO 官方文档交叉验证）

## Standard Architecture

### System Overview

在不改变既有架构红线（adapter thin / core stable / typed contracts）的前提下，新增一个“质量证据平面（Quality Evidence Plane）”，只接在 **test/ops** 层，不侵入 core 业务层。

```
┌──────────────────────────────────────────────────────────────────────────────┐
│ Product Runtime Plane (保持现状)                                             │
├──────────────────────────────────────────────────────────────────────────────┤
│  SvelteKit UI (apps/client/web/app)                                          │
│    ↕ IPC/HTTP                                                                 │
│  Tauri commands (packages/adapters/hosts/tauri/src/commands)                 │
│    ↕ usecases                                                                 │
│  Axum routes (servers/api/src/routes)                                         │
│    ↕ usecases                                                                 │
│  core/usecases + features + contracts + storage adapters                      │
├──────────────────────────────────────────────────────────────────────────────┤
│ Quality Evidence Plane (v0.2.1 新增/强化)                                     │
├──────────────────────────────────────────────────────────────────────────────┤
│  Desktop E2E (WDIO + tauri-driver, Windows/Linux)                            │
│  Web E2E (Playwright, macOS/Windows/Linux)                                   │
│  QA/UAT Manual Evidence (checklist + sign-off artifacts)                     │
│  Regression Fast Lane (changed-slice test selection + failure triage docs)   │
├──────────────────────────────────────────────────────────────────────────────┤
│ Orchestration Plane                                                           │
├──────────────────────────────────────────────────────────────────────────────┤
│  Just stable entrypoints  →  moon task graph  →  GitHub Actions workflows    │
│  Artifacts: JUnit XML / Playwright report / trace / screenshots / videos     │
└──────────────────────────────────────────────────────────────────────────────┘
```

### Component Responsibilities

| Component | Responsibility | Typical Implementation |
|-----------|----------------|------------------------|
| `packages/adapters/hosts/tauri` (existing, modified) | 保持 Tauri 命令薄适配；为测试增加稳定测试钩子（非业务逻辑） | `#[tauri::command]` + state 注入 + feature 调用 |
| `servers/api/src/routes/*` (existing, modified) | 保持 Axum handler 薄适配；返回稳定错误形状供自动化断言 | 路由层 JSON/SSE 包装 usecases |
| `e2e-tests/` (existing, modified) | Desktop runtime E2E（Windows deterministic 为主） | WDIO + tauri-driver + JUnit |
| `apps/client/web/app/tests/e2e` (existing, modified) | Web E2E 作为 macOS 主要自动化证据通道 | Playwright + trace/screenshot/video |
| `.github/workflows/e2e-tests.yml` (existing, modified) | 双测试栈矩阵执行 + 证据上传 | matrix jobs + upload-artifact + summary |
| `qa-uat/` (existing, modified + new files) | 人工 UAT 清单 + 执行证据归档规范 | checklist + runbook + sign-off 模板 |
| `.planning/` and docs/process (new) | 缺陷→复现→修复→回归闭环流程 | bug workflow 文档 + regression policy |

## Recommended Project Structure (v0.2.1 增量)

```text
e2e-tests/
├── fixtures/                           # NEW: deterministic test seeds/session blobs
│   ├── auth/
│   ├── db/
│   └── sync/
├── helpers/                            # EXISTING (扩展): stable selectors/navigation/assert helpers
├── scripts/
│   ├── run-desktop-e2e.mjs             # MODIFIED: deterministic flags + fail-fast diagnostics
│   ├── check-windows-setup.mjs         # EXISTING
│   └── collect-evidence.mjs            # NEW: normalize junit/log/screenshot bundle
├── specs/                              # EXISTING
└── test-results/                       # EXISTING: CI artifact root

apps/client/web/app/tests/e2e/
├── fixtures/                           # NEW: API mock/session fixtures for macOS QA evidence
└── ... existing tests

qa-uat/
├── README.md                           # MODIFIED: pipeline-oriented QA flow
├── uat-checklist.md                    # MODIFIED: map each check to evidence artifact ID
├── evidence/
│   ├── windows/
│   └── macos/                          # NEW: human-signoff payloads (logs, screenshots, notes)
└── workflows/
    ├── bug-intake-and-regression.md    # NEW
    └── release-qa-gate.md              # NEW

.github/workflows/
├── e2e-tests.yml                       # MODIFIED: split deterministic-desktop vs qa-evidence jobs
└── qa-uat-evidence.yml                 # NEW: manual dispatch for UAT evidence capture/sign-off

scripts/
└── test-selection.mjs                  # NEW: changed-files => targeted suite map

moon.yml                                # MODIFIED: repo:test-desktop-deterministic, repo:test-regression-fast
Justfile                                # MODIFIED: just test-desktop-deterministic, just qa-evidence
```

### Structure Rationale

- **不新建业务层目录**：所有新增都在 `tests/qa/workflows/scripts`，避免破坏 runtime 边界。
- **fixtures 单独目录**：把“可重复”作为资产管理，避免 spec 内散落硬编码状态。
- **qa-uat/evidence**：把“是否通过”升级为“可审计证据链”，方便 release gate。

## New vs Modified Boundaries (明确边界)

## ✅ New Components

1. `e2e-tests/fixtures/**`：Windows deterministic 固定输入（账号态、DB seed、sync seed）
2. `e2e-tests/scripts/collect-evidence.mjs`：统一打包 JUnit + logs + screenshots
3. `scripts/test-selection.mjs`：根据 changed paths 做快速回归用例选择
4. `qa-uat/workflows/bug-intake-and-regression.md`：缺陷生命周期标准
5. `qa-uat/workflows/release-qa-gate.md`：发版前 QA gate
6. `.github/workflows/qa-uat-evidence.yml`：macOS + Windows 手工/UAT 证据归档流水线

## ♻️ Modified Components

1. `.github/workflows/e2e-tests.yml`：
   - desktop-e2e 保持 Win/Linux（官方支持）
   - 增加 deterministic job（Windows pinned）
   - artifact 命名标准化（suite/platform/run-id）
2. `e2e-tests/wdio.conf.mjs`：
   - 固定超时、重试、driver 解析策略
   - 失败时附加诊断输出（driver/version/env）
3. `e2e-tests/scripts/run-desktop-e2e.mjs`：
   - 增加 deterministic mode（禁随机、固定顺序、固定 viewport）
4. `apps/client/web/app/playwright.config.ts`：
   - macOS 证据优先 project（desktop-safari/mobile-safari）
   - 保持 `trace: on-first-retry`, `screenshot/video: retain-on-failure`
5. `moon.yml`/`Justfile`：
   - 新增快反馈任务，保留原命令兼容
6. `qa-uat/README.md` + `uat-checklist.md`：
   - 从“说明文档”升级为“证据映射文档”

## Architectural Patterns (for this milestone)

### Pattern 1: Deterministic Test Envelope（确定性测试包络）

**What:** 将“会漂移的环境因素”前置并固定（driver/version/fixture/state/order/timeouts）。
**When to use:** Windows desktop E2E（必须可复现）。
**Trade-offs:** 初期维护 fixture 成本上升，但显著降低 flaky 与误报。

**Example (orchestration):**
```text
repo:test-desktop-deterministic
  -> check-windows-setup
  -> load fixtures (auth/db)
  -> run wdio serially
  -> collect-evidence
```

### Pattern 2: Dual Evidence Channel（自动化 + 手工双证据通道）

**What:**
- Windows: 以 WDIO desktop runtime 自动化为主证据
- macOS: 以 Playwright web-e2e + UAT checklist 为主证据（当前官方限制下）

**When to use:** macOS desktop WebDriver 缺官方稳定支持的阶段。
**Trade-offs:** macOS 桌面自动化覆盖不完整，需要流程补偿（UAT evidence）。

### Pattern 3: Regression Fast Lane（变更感知快速回归）

**What:** changed-path → suite-map → targeted test run；失败后自动升级到 full suite。
**When to use:** 日常 bugfix PR。
**Trade-offs:** 需要维护路径映射，但可显著缩短反馈时间。

## Integration Points (Concrete)

### Tauri Runtime (adapter layer)

- 在 `packages/adapters/hosts/tauri/src/commands/*` 保持“只做协议翻译”。
- 若需测试辅助命令（如 reset fixture state），放在 **test-only command module**，并通过 feature flag 或 `cfg(test)`/env gate 限制在测试环境。
- 不把任何测试条件分支下沉到 core/usecases。

### Axum APIs

- `servers/api/src/routes/*` 输出统一错误体（已有 `{ error: ... }` 形状），供 Playwright/WDIO 稳定断言。
- SSE 路由（agent）保留事件类型（assistant/tool）以便回归检查。

### SvelteKit UI

- 继续使用稳定 data-testid/语义选择器（不要依赖纯样式 class）。
- `src/lib/ipc/*` 双路径（IPC/HTTP）继续保留：
  - Desktop 回归覆盖 IPC path
  - Web/macOS 回归覆盖 HTTP path

### moon / Just / CI

- `Justfile` 保持对外稳定入口（新增命令不重命名旧命令）：
  - `just test-desktop-deterministic`
  - `just test-regression-fast`
  - `just qa-evidence`
- `moon.yml` 负责编排依赖链和输入输出声明，确保缓存和增量执行。
- GitHub Actions 只消费 moon/just 入口，避免 workflow 直接拼装复杂命令。

## Build Order & Dependency Order (推荐)

1. **Phase A: Stabilize Determinism Foundation**
   - `e2e-tests/fixtures` + `run-desktop-e2e` deterministic mode
   - `wdio.conf.mjs` preflight hardening
   - 依赖：无（独立于业务）

2. **Phase B: Evidence Artifact Standardization**
   - `collect-evidence.mjs`
   - workflow artifact naming/retention 统一
   - 依赖：A

3. **Phase C: QA/UAT Evidence Pipeline**
   - `qa-uat-evidence.yml` + checklist 映射 artifact ID
   - 依赖：B

4. **Phase D: Fast Regression Loop**
   - `scripts/test-selection.mjs`
   - `repo:test-regression-fast` + fallback full suite
   - 依赖：A（可并行部分实现）

5. **Phase E: Bug Workflow Institutionalization**
   - `bug-intake-and-regression.md` + PR template linkage
   - 依赖：C + D

## Data Flow (质量数据流)

### Deterministic Windows E2E Flow

```text
PR/Push
  -> just test-desktop-deterministic
  -> moon repo:test-desktop-deterministic
  -> WDIO + tauri-driver + fixtures
  -> junit/xml + logs + screenshots
  -> upload-artifact + summary
```

### macOS + Windows QA/UAT Evidence Flow

```text
Release candidate / workflow_dispatch
  -> web-e2e matrix (macOS + Windows)
  -> manual UAT checklist execution
  -> evidence folder bundle (screenshots/logs/sign-off)
  -> qa-uat-evidence artifact
  -> release gate decision
```

### Bug-fix Feedback Loop

```text
Issue labeled bug
  -> reproduce with fixture id
  -> run test-regression-fast
  -> fix in adapter/route/ui slice
  -> rerun targeted + required smoke/full gate
  -> attach evidence links in issue/PR
```

## Rollout Strategy (Low Blast Radius)

1. **Shadow mode first**：新 deterministic 任务先不作为 required check，仅并行观测 1-2 周。
2. **No behavior change in runtime**：除 test-only hooks 外，不改 production command/handler 语义。
3. **Incremental enforcement**：
   - Week 1: artifact 标准化
   - Week 2: regression-fast required
   - Week 3: deterministic desktop required（Windows）
4. **Fallback always available**：保留 `repo:test-desktop` 旧路径，故障时可快速切回。

## Risks & Rollback Paths

### Risk 1: Windows driver/version drift 导致假失败
**Mitigation:** preflight 输出 Edge/WebDriver 版本；固定安装步骤；失败自动附环境快照。
**Rollback:** 将 required check 从 `deterministic` 回退到现有 `test-desktop`。

### Risk 2: macOS 无官方桌面 WebDriver 支持造成覆盖缺口
**Mitigation:** macOS 采用 Playwright + UAT 证据组合；明确不是 desktop runtime parity。
**Rollback:** 若 macOS job 噪声高，仅保留 Windows/Linux 自动化 + 人工 macOS UAT。

### Risk 3: 快速回归策略漏测
**Mitigation:** `test-regression-fast` 失败或 mapping miss 时自动升级 full suite。
**Rollback:** 立即禁用 path selection，恢复 full suite 为默认。

## Anti-Patterns to Avoid

1. **在 core/usecases 注入测试分支逻辑** → 应放在 test harness/fixtures/adapters。
2. **workflow 里直接硬编码业务命令链** → 必须经 Just/moon 稳定入口。
3. **只保留 pass/fail 不保留证据** → 必须上传结构化 artifacts。
4. **把社区实验能力当官方支持**（如第三方 macOS Tauri WebDriver）→ 仅可作为 LOW confidence 试验通道。

## Sources

### HIGH confidence
- Tauri WebDriver docs (v2): https://v2.tauri.app/develop/tests/webdriver/
- Tauri CI guidance for WebDriver: https://v2.tauri.app/develop/tests/webdriver/ci/
- Playwright docs (trace/view/video/reporter): https://playwright.dev/docs/trace-viewer, https://playwright.dev/docs/videos
- WebdriverIO docs (GitHub Actions/JUnit reporter): https://webdriver.io/docs/githubactions/, https://webdriver.io/docs/bamboointegration/
- Repo current state:
  - `.github/workflows/e2e-tests.yml`
  - `e2e-tests/wdio.conf.mjs`
  - `e2e-tests/scripts/run-desktop-e2e.mjs`
  - `apps/client/web/app/playwright.config.ts`
  - `qa-uat/README.md`, `qa-uat/uat-checklist.md`
  - `apps/client/native/src-tauri/src/lib.rs`
  - `packages/adapters/hosts/tauri/src/commands/*`
  - `servers/api/src/routes/*`
  - `moon.yml`, `Justfile`
  - `docs/blueprints/agent-native-starter-v1/04-contracts-typegen-and-boundaries.md`
  - `docs/blueprints/agent-native-starter-v1/05-runtime-features-and-adapters.md`

### LOW confidence (标记为实验，不纳入 v0.2.1 主方案)
- 社区 macOS Tauri WebDriver 尝试（非官方稳定）：https://github.com/danielraffel/tauri-webdriver

---
*Architecture research for: milestone v0.2.1 quality hardening*
*Researched: 2026-04-06*
