# Feature Research

**Domain:** Cross-platform desktop product quality hardening (Windows + macOS focus)
**Researched:** 2026-04-06
**Confidence:** MEDIUM

## Feature Landscape

### Table Stakes (Users Expect These)

Features users assume exist. Missing these = release risk, not innovation.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| Windows desktop E2E baseline as merge gate | Windows is primary desktop install base; mature teams treat Win E2E as release blocker | MEDIUM | 当前仓库已有 WDIO + tauri-driver + Windows CI job；v0.2.1 应从“能跑”升级为“强制通过的 required check”。依赖 `e2e-tests/` 现有基线与 `msedgedriver` 供应稳定性。 |
| Windows + macOS full QA/UAT flow (manual + automated mix) | 跨平台产品必须证明“关键用户路径”在双平台可用 | HIGH | 当前 desktop WebDriver 在 macOS 不官方支持（Tauri docs），因此 macOS 需要“Web E2E + 手工 UAT + 构建/签名验证”组合门禁，而非强行同构 WDIO。 |
| Bug lifecycle governance (状态、严重级、SLA、退出条件) | 成熟产品都把 bug 从“发现→分级→修复→回归→关闭”制度化 | MEDIUM | 现有仓库已有 QA 文档与 checklist，但缺统一状态机与 SLA。应落到 issue template / labels / PR 规则 / release checklist。 |
| Regression safety net on every fix | 质量硬化核心是“每个修复都不会破坏已修功能” | HIGH | 依赖测试分层：Rust/Vitest/Playwright/WDIO。每个 P0/P1 缺陷必须补最小回归用例（自动优先）。 |
| Release quality bars with measurable pass/fail thresholds | 没有量化门槛就无法稳定放行 | MEDIUM | 需要把“通过标准”写进 REQUIREMENTS：测试通过率、P0/P1 未关闭数、跨平台 UAT sign-off、崩溃率/阻断缺陷阈值。 |

### Differentiators (Competitive Advantage)

Features that set the product apart. Not required, but valuable.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| Risk-based test selection + bug→test linkage | 修复效率更高：按风险优先跑关键路径，并确保每个高优 bug 有回归映射 | HIGH | 可通过 bug label 映射到测试目录/标签（如 auth, counter, agent）；后续可接 moon task graph 做选择性回归。 |
| Flaky-test governance (quarantine + repair SLA) | 提高 CI 信号可信度，减少“假红/假绿” | HIGH | Playwright/WDIO 都支持 retries；差异化点是“可观测治理”：隔离名单、失败预算、超期强制修复，而不是无限重试。 |
| Evidence-rich quality reports per release | 让发布评审从主观变客观（trace/video/junit + UAT record） | MEDIUM | 现有 Playwright 已有 trace/video/screenshot on failure；补齐 WDIO 失败证据归档与统一发布摘要模板。 |
| Desktop-host-specific regression packs (IPC/auth/store/offline) | 针对 Tauri 特有风险（非纯 Web）建立更强护栏 | HIGH | 依赖现有 Tauri IPC 双路径与 auth adapter 改造进度（AUTH-01）。这类用例是跨端桌面产品的“真实护城河”。 |

### Anti-Features (Commonly Requested, Often Problematic)

Features that seem good but create problems.

| Feature | Why Requested | Why Problematic | Alternative |
|---------|---------------|-----------------|-------------|
| “100% 全自动 UAT，无人工签核” | 想节省 QA 人力 | macOS desktop driver 现实约束 + 视觉/交互类问题难以全自动覆盖 | 采用“自动化回归 + 人工场景 UAT”双轨，保留关键人工签核点 |
| “失败就无限重试直到绿” | 想快速恢复 CI 绿灯 | 会掩盖真实回归，制造质量幻觉 | 限制重试次数（如 CI 2 次），并对 flaky 测试进入隔离/修复流程 |
| “一个超大 E2E 套件包打天下” | 维护看似简单 | 执行慢、定位难、跨平台波动大 | 分层策略：smoke（必跑）+ core regression（必跑）+ extended（夜间/候选发布） |
| “把每个低优 bug 都挡发布” | 追求零缺陷 | 发布吞吐量崩溃，团队被噪音拖垮 | 引入 severity+SLA：仅 P0/P1 阻断发布，P2/P3 进计划窗口 |
| “强制 macOS 走与 Windows 同一 WDIO 路线” | 追求框架统一 | 与 Tauri 官方支持现状冲突，维护成本高 | macOS 用 Web E2E + 手工桌面 UAT + 构建验收，等待官方能力成熟 |

## Feature Dependencies

```
[Release Quality Bars]
    └──requires──> [Bug Lifecycle Governance]
                         └──requires──> [Issue states/severity/SLA conventions]

[Regression on Every Fix]
    └──requires──> [Windows Desktop E2E Baseline as Required Check]
                         └──requires──> [Stable tauri-driver + msedgedriver setup]

[Windows + macOS Full QA/UAT]
    └──requires──> [Platform-specific execution model]
                         ├──Windows: [WDIO Desktop E2E + UAT]
                         └──macOS:  [Playwright Web E2E + Desktop UAT + Build verification]

[Desktop-host-specific regression pack]
    └──requires──> [AUTH-01 and host adapter wiring maturity]

[Unlimited retries / no manual sign-off]
    ──conflicts──> [Reliable release gate signal]
```

### Dependency Notes

- **Release Quality Bars requires Bug Lifecycle Governance:** 没有统一缺陷状态与优先级，无法形成可执行的放行判断。
- **Regression on Every Fix requires Windows baseline gate:** 当前桌面自动化核心能力在 Windows/Linux 更成熟，先把 Windows gate 固化最具性价比。
- **Windows + macOS Full QA/UAT requires platform-specific model:** 基于 Tauri WebDriver 官方支持边界，macOS 需要差异化策略，不宜一刀切。
- **Desktop-host-specific regression depends on AUTH-01:** auth 适配链路未完全闭环前，相关回归用例稳定性与覆盖都会受限。

## MVP Definition

### Launch With (v0.2.1 hardening baseline)

- [ ] **Windows Desktop E2E required check** — PR 到受保护分支必须通过（含 junit artifact）
- [ ] **Windows + macOS release UAT checklist** — 双平台关键路径手工签核记录必填
- [ ] **Bug lifecycle policy** — 统一状态、严重级、SLA、回归要求、关闭定义
- [ ] **Regression-on-fix rule** — 每个 P0/P1 缺陷至少新增 1 个自动回归测试（无法自动化时记录原因与补偿计划）
- [ ] **Release gate dashboard** — 发布前自动汇总测试结果 + 未解决缺陷 + UAT 签核

### Add After Validation (v0.2.1.x)

- [ ] **Flaky quarantine mechanism** — 建立隔离列表与修复 SLA（避免阻断主干）
- [ ] **Risk-based selective regression** — 依据改动范围自动选择回归集
- [ ] **Failure evidence unification** — Playwright/WDIO 失败证据统一索引（trace/video/screenshot/log）

### Future Consideration (v0.3+)

- [ ] **macOS desktop E2E automation parity** — 等 Tauri/macOS driver 支持成熟后再推进
- [ ] **Visual regression + accessibility gate** — 作为质量上限提升，不阻塞本里程碑主线
- [ ] **Reliability KPIs automation**（逃逸缺陷率、MTTR、回归漏检率）— 需要连续迭代数据沉淀

## Feature Prioritization Matrix

| Feature | User Value | Implementation Cost | Priority |
|---------|------------|---------------------|----------|
| Windows desktop E2E required check | HIGH | MEDIUM | P1 |
| Windows + macOS full QA/UAT flow | HIGH | HIGH | P1 |
| Bug lifecycle governance policy | HIGH | MEDIUM | P1 |
| Regression-on-fix enforcement | HIGH | HIGH | P1 |
| Release quality bars dashboard | HIGH | MEDIUM | P1 |
| Flaky quarantine governance | MEDIUM | HIGH | P2 |
| Risk-based selective regression | MEDIUM | HIGH | P2 |
| macOS desktop E2E parity | MEDIUM | HIGH | P3 |

**Priority key:**
- P1: Must have for v0.2.1 hardening gate
- P2: Should have shortly after baseline stabilizes
- P3: Nice to have, dependent on external ecosystem maturity

## Measurable Acceptance Expectations (Release Gate Quality Bars)

建议直接转写到 `REQUIREMENTS.md` 的“验收标准”条款：

1. **Merge Gate (PR级)**
   - 受保护分支启用 required status checks（至少：Rust tests、Vitest、Playwright smoke、Windows WDIO desktop E2E）
   - 任一 required check fail → 禁止合并

2. **Bug Gate (Release级)**
   - P0：0 个未关闭
   - P1：0 个未关闭（允许“有缓解+业务批准”例外，但需记录）
   - 所有“本迭代引入且已修复”的 P0/P1：100% 具备回归验证记录（自动或人工）

3. **Cross-platform QA/UAT Gate**
   - Windows：自动 E2E 全绿 + 手工 UAT 签核
   - macOS：Web E2E 关键路径全绿 + 桌面手工 UAT 签核 + 构建可启动验证
   - 双平台均需保留可追溯 evidence（报告/截图/日志）

4. **Flakiness Gate**
   - CI 重试上限固定（例如 2）；禁止临时提高重试作为常态修复
   - 被标记 flaky 的测试需进入隔离清单并设置修复截止期

5. **Operational Gate**
   - 发布说明必须包含：缺陷清单（按严重级）、已知风险、回滚路径、验证摘要

## Competitor / Mature Pattern Snapshot

| Capability | Mature Pattern (2026) | Our Approach for v0.2.1 |
|-----------|-------------------------|--------------------------|
| Branch safety | Protected branches + required checks + merge queue | 先启用 required checks；PR 量上来后再启 merge queue |
| E2E reliability | Limited retries + rich failure artifacts + flaky governance | 保留 retries（CI=2）+ 强制证据归档 + 补 flaky 流程 |
| Defect lifecycle | Explicit status/priority/resolution + regression tracking | 在 issue/PR/release 模板中固化状态机与回归责任 |
| Cross-platform quality | Platform-specific validation model, not one-size-fits-all | Windows 重自动化；macOS 走混合验证模型 |

## Sources

### High confidence (official docs / repo facts)
- Tauri WebDriver docs (desktop support scope, tauri-driver usage): https://v2.tauri.app/develop/tests/webdriver/
- Context7: `/tauri-apps/tauri-docs` (WebDriver examples, CI notes, platform limits)
- Playwright retries and flaky categorization: https://playwright.dev/docs/test-retries
- Context7: `/microsoft/playwright` (CI retries / trace-on-retry / sharding patterns)
- WebdriverIO retry and spec-level retry guidance: https://webdriver.io/docs/retry/
- Context7: `/webdriverio/webdriverio` (reporters/retry/CI patterns)
- GitHub protected branches and required checks: https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-protected-branches/about-protected-branches
- GitHub merge queue: https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/configuring-pull-request-merges/managing-a-merge-queue
- Sentry issue states + regressed model: https://docs.sentry.io/product/issues/states-triage/
- Jira statuses/priorities/resolutions (workflow baseline): https://support.atlassian.com/jira-cloud-administration/docs/what-are-issue-statuses-priorities-and-resolutions/

### Repository evidence (current codebase state)
- `.github/workflows/e2e-tests.yml` (Windows/Linux desktop E2E, macOS web E2E)
- `e2e-tests/README.md` (macOS desktop skip by default; Windows Edge driver requirement)
- `e2e-tests/wdio.conf.mjs` (Windows native driver checks, JUnit reporter)
- `apps/client/web/app/playwright.config.ts` (CI retries, trace/screenshot/video on failure)
- `qa-uat/README.md`, `qa-uat/uat-checklist.md` (现有 UAT 与质量条目)

### Low confidence / needs follow-up
- Exa ecosystem scan for generic “2026 QA practices” reached rate limit and returned mostly vendor blogs；本文件未将其作为关键结论依据。

---
*Feature research for: v0.2.1 quality hardening milestone*
*Researched: 2026-04-06*
