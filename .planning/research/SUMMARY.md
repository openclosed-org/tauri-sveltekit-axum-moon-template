# Project Research Summary

**Project:** tauri-sveltekit-axum-moon-template（milestone v0.2.1）
**Domain:** 跨平台桌面产品质量硬化（Windows 深度 E2E + Windows/macOS QA/UAT + 缺陷闭环）
**Researched:** 2026-04-06
**Confidence:** MEDIUM-HIGH

## Executive Summary

本次研究结论非常一致：v0.2.1 不应重构主架构，而应在现有 `Tauri v2 + SvelteKit 2 + Svelte 5 + Axum + moon + Just` 的 brownfield 基础上，构建“质量证据平面”。核心策略是**Windows 走原生桌面 E2E（WDIO + tauri-driver）作为硬门禁，macOS 走 Web E2E + 手工 UAT + 构建可启动验证**，避免与当前 Tauri 平台能力边界冲突。

推荐落地方式是“先稳定、再证据、后治理”：先做确定性测试包络（版本钉住、fixture 隔离、预检与环境快照），再统一 artifact 与发布前证据链，最后制度化 bug 生命周期（状态/严重级/SLA/回归义务）。这条路径最符合仓库现状（已有 WDIO、Playwright、qa-uat、moon/Just 入口），改动局部、可回滚、可验证。

主要风险不是“工具不够”，而是“信号失真”：无限重试掩盖 flaky、时间等待与脆弱选择器导致跨 OS 不稳定、`*-latest` runner 漂移制造假回归、workflow 触发遗漏导致门禁空转。应通过 first-run pass rate、固定 required checks、失败证据强制归档、平台差异化 gate 来降低发布误判概率。

## Key Findings

### Recommended Stack

v0.2.1 的栈策略是“增强测试与治理链路，不改运行时主栈”。优先级最高的增量是：固定并强化现有 WDIO/Playwright 配置、升级并标准化 GitHub Actions 编排与 artifacts、补齐 triage 自动化脚本能力，同时保持 `Just -> moon` 作为唯一稳定入口，避免脚本分叉。

**Core technologies:**
- **WebdriverIO + tauri-driver（9.27.x）**：Windows 桌面原生 E2E 主引擎 — 与仓库现状一致、迁移成本最低、对 Tauri runtime 覆盖最贴近。
- **Playwright Test（1.58.x）**：跨 OS Web E2E/UAT 主引擎 — 支持 shard/blob/report merge，适合 Win/macOS 扩展。
- **GitHub Actions matrix + workflow_run**：跨平台执行与失败后闭环编排 — 统一测试工件、支撑 triage 自动化。
- **actions/github-script@v8**：轻量 defect triage 自动化 — 直接调用 GitHub API，无需引入重型平台。
- **moon + Just（任务增强）**：统一执行入口与依赖图 — 保持 brownfield 兼容，避免新增第二套编排。

**Stack additions（actionable）:**
- 新增 `@wdio/html-reporter` 提升桌面 E2E 可读性（与 JUnit 并存）。
- Playwright CI 启用 blob reporter + merge job（跨 shard / 跨 job 汇总）。
- 可选 `fast-xml-parser` 用于 JUnit 结构化摘要（供 github-script 自动建单/去重）。
- Actions 统一升级到 artifact v4/v5 与 node/checkout v5（同批验证缓存行为）。

### Expected Features

**Must have（table stakes / v0.2.1 必达）:**
- Windows desktop E2E 成为 required check（PR 合并门禁）。
- Windows + macOS QA/UAT 双平台签核（自动化 + 人工混合）。
- Bug 生命周期治理（状态、严重级、SLA、关闭定义、回归责任）。
- Regression-on-fix 规则（P0/P1 修复必须补回归测试或书面补偿方案）。
- 发布质量看板（测试结果、未决缺陷、UAT evidence 汇总）。

**Should have（v0.2.1.x 差异化增强）:**
- Flaky 治理（隔离清单 + 修复 SLA + 失败预算）。
- 风险驱动选择性回归（changed-path -> suite-map）。
- Playwright/WDIO 失败证据统一索引（trace/video/screenshot/log）。

**Defer（v0.3+）:**
- macOS 桌面 E2E 与 Windows 同级自动化（等待生态成熟）。
- 视觉回归与 a11y gate 作为主门禁。
- 可靠性 KPI 全自动化（需要持续数据沉淀）。

### Architecture Approach

架构建议明确：新增“Quality Evidence Plane”，仅落在 `tests/qa/workflows/scripts/CI`，**不侵入 core/usecases**。运行时保持 adapter-thin/core-stable；测试增强通过 fixture、deterministic mode、evidence bundling、workflow 标准化来完成。

**Major components（integration focus）:**
1. **Desktop E2E lane（`e2e-tests/`）** — Windows 确定性执行、driver 预检、JUnit+诊断证据。
2. **Web E2E lane（`apps/client/web/app/tests/e2e`）** — macOS/Windows 跨平台自动化证据主通道。
3. **QA/UAT evidence lane（`qa-uat/` + workflows）** — 手工签核与 artifact ID 映射，形成可审计链路。
4. **Orchestration lane（`Justfile` + `moon.yml` + GH Actions）** — 稳定入口、任务图、矩阵与工件归档。

### Critical Pitfalls

**Top watch-outs（必须写入 requirements/roadmap 的防线）:**

1. **重试掩盖不稳定（green but unstable）** — 以 first-run pass rate 和 flaky 预算为准，不以“最终重试变绿”判定质量。
2. **时间等待/脆弱选择器导致跨平台抖动** — 禁用固定 sleep，统一语义选择器与 web-first 断言。
3. **测试状态泄漏（共享账号/可变 fixture）** — worker 级隔离账号与 deterministic seed/reset。
4. **Runner/tool 漂移触发假回归** — release gate lane 固定 OS 与工具版本，另设 latest canary。
5. **门禁触发配置缺口** — required checks 必须覆盖 `pull_request` 与 `merge_group`，并定期审计分支保护。

## Implications for Roadmap

基于依赖关系与风险最小化，建议 5 个阶段（先基础、后证据、再治理）：

### Phase 1: Deterministic Foundation
**Rationale:** 不先稳定环境，后续门禁会持续假红/假绿。  
**Delivers:** Windows deterministic desktop E2E（fixture、preflight、固定 timeout/retry/顺序）。  
**Addresses:** Windows desktop E2E required check（table stake）。  
**Avoids:** 环境漂移、状态泄漏、重试掩盖问题。

### Phase 2: Artifact & Evidence Standardization
**Rationale:** 没有统一证据，bug-loop 和 release gate 无法审计。  
**Delivers:** `collect-evidence`、JUnit/trace/video/log 统一命名与归档、CI summary 模板。  
**Uses:** upload/download-artifact v4/v5、Playwright blob merge、WDIO reporters。  
**Implements:** Quality Evidence Plane 的证据层。

### Phase 3: Cross-platform QA/UAT Gate
**Rationale:** 需要在平台能力差异下建立可执行的放行标准。  
**Delivers:** Windows（desktop E2E + UAT）与 macOS（web E2E + desktop UAT + build verify）双轨 gate。  
**Addresses:** Windows+macOS full QA/UAT、release quality bars。  
**Avoids:** 平台盲区与“Windows 过、macOS 未验证”风险。

### Phase 4: Bug Lifecycle & Regression Enforcement
**Rationale:** 质量硬化要从“一次通过”升级为“持续闭环”。  
**Delivers:** 缺陷状态机/SLA、P0/P1 回归义务、issue/PR/release 模板联动、triage 自动化。  
**Addresses:** bug lifecycle governance、regression-on-fix。  
**Avoids:** 缺陷反复回归、无法复现、责任不清。

### Phase 5: Fast Regression Lane & Flaky Governance
**Rationale:** 基线稳定后再做效率优化，防止过早复杂化。  
**Delivers:** changed-path 选择性回归、失败自动升级 full suite、flaky quarantine/SLA。  
**Addresses:** v0.2.1.x differentiators（risk-based selection、flaky 治理）。  
**Avoids:** PR 反馈过慢与重试依赖。

### Phase Ordering Rationale

- 先做确定性与版本钉住，再做门禁治理，避免“制度先行但信号不可信”。
- 证据标准化必须早于缺陷闭环，否则 triage/回归缺乏可追踪输入。
- 跨平台 gate 在 Phase 3 落地，确保 roadmap 对 macOS 使用差异化策略而非错误同构。
- 快速回归和 flaky 优化放后置，以避免在基础不稳时引入复杂选择逻辑。

### Research Flags

需要 `/gsd-research-phase` 深挖：
- **Phase 3（Cross-platform QA/UAT Gate）**：需确认 macOS 当前可自动化边界、签名/验包流程细节与证据最小集合。
- **Phase 5（Fast Regression Lane）**：changed-path -> suite-map 的映射策略和误判回退机制需要样本验证。

标准模式可直接规划（可跳过额外 research-phase）：
- **Phase 1（Deterministic Foundation）**：WDIO/Playwright/GitHub Actions 官方模式成熟，仓库已有基础。
- **Phase 2（Artifact Standardization）**：artifact/report merge/junit summary 已有稳定行业实践。
- **Phase 4（Bug Lifecycle Enforcement）**：issue 状态机与 branch protection 为成熟治理范式。

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | 以官方文档 + Context7 + 仓库现状三方验证，且与现有实现高度兼容。 |
| Features | MEDIUM | 结论可执行，但部分“2026 行业趋势”来源受限，优先依据官方与仓库事实。 |
| Architecture | HIGH | 明确基于 repo 现状（目录、workflow、任务编排）给出增量边界，风险可控。 |
| Pitfalls | MEDIUM-HIGH | 风险模式清晰且可观测；少量外部生态（macOS desktop driver）仍在演进。 |

**Overall confidence:** MEDIUM-HIGH

### Gaps to Address

- **macOS desktop E2E 官方能力演进**：在 Phase 3 执行前二次验证 Tauri 生态现状，避免过度承诺自动化覆盖。
- **Flaky 预算阈值初始值**：需基于 1-2 周基线数据确定（first-run pass rate、隔离触发阈值）。
- **triage 自动建单去重策略**：需定义稳定去重 key（test + platform + commit window），防止噪音 issue 泛滥。
- **签名/公证 release-like 工件的 QA 节点**：需与发布流程确认最早可插入时机，避免末端爆雷。

## Sources

### Primary (HIGH confidence)
- Tauri v2 WebDriver / CI docs — 平台支持边界、tauri-driver 官方路径。
- Playwright official docs + Context7 `/microsoft/playwright` — retries/sharding/blob merge/artifacts/best practices。
- WebdriverIO official docs + Context7 `/webdriverio/webdriverio` — reporter/retry/CI patterns。
- GitHub Actions official docs + Context7 `/websites/github_en_actions` — matrix/workflow_run/merge_group/required checks。
- Actions `upload-artifact` / `github-script` docs — artifact 聚合与自动 triage 能力。
- Repository evidence — `.github/workflows/e2e-tests.yml`, `e2e-tests/*`, `apps/client/web/app/playwright.config.ts`, `moon.yml`, `Justfile`, `.tool-versions`, `qa-uat/*`。

### Secondary (MEDIUM confidence)
- `tauri-apps/tauri-action` 文档与仓库实践（用于发布构建一致性，不作为主测试框架）。

### Tertiary (LOW confidence)
- 社区 macOS Tauri WebDriver 尝试（实验性，v0.2.1 不纳入主方案）。

---
*Research completed: 2026-04-06*
*Ready for roadmap: yes*
