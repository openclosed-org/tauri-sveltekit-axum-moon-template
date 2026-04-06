# Stack Research

**Domain:** v0.2.1 质量硬化（Windows 深度 E2E + Windows/macOS 全覆盖 QA/UAT/E2E + 持续缺陷闭环）  
**Researched:** 2026-04-06  
**Confidence:** HIGH（核心结论由 Context7 + 官方文档 + 仓库现状三方交叉验证）

## Recommended Stack

> 约束声明：不重选主栈。维持 `Tauri v2 + SvelteKit 2 + Svelte 5 + Axum + Bun + moon + proto/mise + Just`，只做测试/质量链路增强。

### Core Technologies

| Technology | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| WebdriverIO + tauri-driver（保留） | WDIO `^9.27`；tauri-driver 最新锁定安装 | **Windows 桌面原生 E2E 主引擎**（Tauri WebDriver 协议） | Tauri v2 官方 WebDriver 指南与 CI 示例仍以 tauri-driver 为标准路径；你们仓库已落地 `e2e-tests/wdio.conf.mjs`，迁移成本最低且最稳。 |
| Playwright Test（强化） | `1.58.x`（当前仓库已 1.58.2） | **跨 OS（Windows/macOS）Web 层 E2E/UAT 主引擎** + 报告汇总 | 现有配置已开启 trace/video/screenshot；官方文档支持分片、blob 报告、合并报告，适合扩大到全矩阵 CI。 |
| GitHub Actions Matrix + workflow_run | `actions/upload-artifact@v4`、`download-artifact@v5`、`checkout@v5`、`setup-node@v5` | **跨平台执行编排 + 测试工件汇聚 + 失败后续自动化** | 适合将 Windows/macOS QA/UAT/E2E 做成可重复流水线，并把“失败→工单/分流→回归”串成自动闭环。 |
| actions/github-script | `@v8` | **自动 triage（创建/更新 issue、加标签、回帖）** | 官方维护，Node 24 runtime，直接调用 GitHub API；比引入第三方“全家桶”更可控。 |
| tauri-apps/tauri-action（仅发布/验包链路） | `action-v0.6.x`（文档示例标签仍可用 `@v1`） | **Windows/macOS 产物打包验证与发布一致性** | 不是测试框架，而是把“可测试构建”与“可发布构建”统一，减少“测试过但发布坏了”的偏差。 |

### Supporting Libraries

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| @wdio/html-reporter（新增） | `^9.27` | 本地与 CI 可读性更高的桌面 E2E 报告 | 需要快速人工复盘失败 UI 流程时（与 junit 并存）。 |
| @wdio/junit-reporter（保留并固定） | `^9.27` | CI 结构化结果输出 | 供 Actions 汇总、后续 triage 解析、PR 状态检查使用。 |
| Playwright blob reporter（配置层新增） | 内置 | 跨 shard / 跨 job 报告合并 | 当 web E2E 扩展成分片矩阵（Win/macOS）时必须开启。 |
| fast-xml-parser（可选新增，轻量） | `^5` | 解析 JUnit XML 生成失败摘要（供 github-script） | 若你们要做“自动 issue 内容结构化”但不想上外部 SaaS。 |
|（可选）ctrf-io/github-test-reporter | `@v1` | 汇总多框架测试结果到 Job Summary / PR 评论 | 仅在你们希望“现成仪表化+flaky 洞察”时启用；非必需。 |

### Development Tools

| Tool | Purpose | Notes |
|------|---------|-------|
| moon（新增 repo 任务） | 统一质量任务图（desktop/web/qa/triage/regression） | 新增 `repo:qa-ci`, `repo:qa-desktop-win`, `repo:qa-web-matrix`, `repo:triage-sync`, `repo:regression`。 |
| Just（新增稳定入口） | 人类/Agent 的单命令入口 | 新增 `just qa`, `just qa-win`, `just qa-macos`, `just regression`, `just triage` 映射 moon。 |
| mise/proto（版本钉住） | 避免 runner 漂移导致假失败 | 保持 Node 25.9 / Bun 1.3.11 / Rust 1.94.1；仅在统一验证后升级。 |

## Installation

```bash
# Desktop E2E reports
bun add -d --cwd e2e-tests @wdio/html-reporter@^9.27.0

# Optional: parse junit for scripted triage
bun add -d fast-xml-parser@^5

# Optional: unified GH report layer
bun add -d ctrf @ctrf/junit-to-ctrf
```

## Alternatives Considered

| Recommended | Alternative | When to Use Alternative |
|-------------|-------------|-------------------------|
| WDIO + tauri-driver（桌面） | 只用 Playwright 覆盖所有桌面场景 | 仅当你把目标降级为“WebView 内网页行为”而非 Tauri 原生窗口/驱动链路。当前里程碑不建议。 |
| github-script（轻量自建 triage） | 完整测试管理平台（TestRail/Xray/Allure TestOps） | 团队规模扩大、需要审批流/报表治理时再引入；v0.2.1 会显著过重。 |
| GitHub Actions 原生矩阵+工件 | 额外引入 Jenkins/Buildkite | 仅在组织级合规或并发规模超过 GitHub Hosted runner 能力时。 |

## What NOT to Use

| Avoid | Why | Use Instead |
|-------|-----|-------------|
| **为 macOS 强行跑 tauri-driver 桌面 WebDriver** | 仓库当前脚本已明确跳过，且 Tauri 官方 CI 示例只覆盖 Linux/Windows；会制造大量假阴性 | macOS 使用 Playwright Web E2E + 构建验包/UAT 清单。 |
| 立刻引入 Allure/Jira 双向同步 + 多个 SaaS | 在当前规模下维护成本大于收益，且与 moon/Just 入口冲突 | 先用 JUnit + GH Summary + github-script 闭环，后续再演进。 |
| 新增第二套任务编排（纯 npm scripts 直跑） | 会破坏你们已建立的 `Just -> moon` 稳定入口 | 所有新增能力都挂到 `repo:*` 任务，再由 just 暴露。 |
| 为“观感先进”引入视觉回归平台（Percy/Chromatic）作为本里程碑核心 | 你们当前核心痛点是跨 OS 稳定性与缺陷闭环，不是视觉 diff | 先把功能回归闭环跑通，再评估视觉回归。 |

## Stack Patterns by Variant

**If 目标是 Windows 深度自动化（桌面原生路径）:**
- Use WDIO + tauri-driver + msedgedriver（已在 `e2e-tests` 实装）
- Because 这条链路直接覆盖 Tauri 桌面运行时与 WebDriver 代理，是当前最贴合“原生壳+WebView”故障模型的自动化路径。

**If 目标是 Windows + macOS 全覆盖回归速度:**
- Use Playwright（Web E2E）矩阵 + shard + blob merge + artifacts
- Because Playwright 在跨 OS 并行与报告合并上成熟，能把 macOS 纳入同等质量门禁。

**If 目标是 bug report → triage → fix → regression 闭环:**
- Use JUnit/JSON 结果 + workflow_run + github-script（自动创建/更新 issue + 标签 + 失败摘要）
- Because 无需重平台即可获得“失败工单化”和“回归证据链接”。

## Version Compatibility

| Package A | Compatible With | Notes |
|-----------|-----------------|-------|
| `@playwright/test@1.58.x` | Node 20+（仓库 Node 25.9） | 已在仓库运行；建议继续 pin 次版本带。 |
| `@wdio/*@9.27.x` | Node 18+（仓库 Node 25.9） | 现有 `^9.20` 实际锁文件已到 `9.27.0`，建议显式对齐。 |
| `actions/github-script@v8` | Runner `>=2.327.1` | 官方声明 Node 24 runtime；GitHub Hosted runner满足。 |
| `actions/upload-artifact@v4` + `download-artifact@v5` | GitHub Actions | v4/v5 组合是当前官方主线；支持 retention 与并行工件聚合。 |

## Integration with moon / Just / proto(mise)

推荐最小增量任务图（不破坏既有结构）：

1. **moon 根任务新增**
   - `repo:qa-desktop-win` → `bun run --cwd e2e-tests test:ci`（仅 Windows runner）
   - `repo:qa-web-matrix` → `bun run --cwd apps/client/web/app test:e2e --project=desktop-chrome`（可分片）
   - `repo:qa-report-merge` → 合并 Playwright blob + 汇总 JUnit
   - `repo:triage-sync` → 读取失败报告，执行 github-script/gh issue 自动分流
   - `repo:regression` → 对“已标记 bug”场景回放（先从 tag 或 spec list 实现）

2. **Just 暴露稳定入口**
   - `just qa` -> `moon run repo:qa-web-matrix` + 条件触发 `repo:qa-desktop-win`
   - `just qa-win` / `just qa-macos`
   - `just triage` -> `moon run repo:triage-sync`
   - `just regression` -> `moon run repo:regression`

3. **proto/mise 版本策略**
   - 继续使用 `.tool-versions` 当前版本（Rust 1.94.1 / Node 25.9 / Bun 1.3.11）
   - CI 与本地统一读取，禁止 workflow 内“临时 latest”漂移

## Migration & Risk Notes

- **低风险（建议本里程碑直接做）**
  - 固化 WDIO 版本到 `9.27.x`；补 HTML reporter；补 Actions 工件与 Summary。
  - Playwright 改 CI 为 blob + merge job（本地可保留 html/junit/json）。

- **中风险（需要一次性治理）**
  - 将 `e2e-tests.yml` 中旧 action 版本（如 checkout/setup-node v4）升级到 v5 时，需同批验证 cache 行为与运行时兼容。
  - triage 自动建单会引入“噪音 issue”风险，需先设计去重 key（测试名+平台+commit 窗口）。

- **明确不做（v0.2.1）**
  - 不引入新的测试编排平台、不改主架构、不上重型测试管理 SaaS。

## Sources

- **Context7 `/websites/v2_tauri_app`** — Tauri v2 WebDriver 与 CI 指南（Linux/Windows、tauri-driver 独立安装）【HIGH】  
- **Context7 `/microsoft/playwright`** + 官方文档 https://playwright.dev/docs/test-sharding — shard/blob/merge/reporter/retry/traces【HIGH】  
- **Context7 `/websites/github_en_actions`** + 官方文档（matrix/workflow_run/job summary）【HIGH】  
- **Context7 `/actions/upload-artifact`** — v4 行为、retention、overwrite、artifact 输出【HIGH】  
- **Context7 `/actions/github-script`** + 官方仓库 README（v8, Node24, runner 要求）【HIGH】  
- **Context7 `/tauri-apps/tauri-action`** + 官方仓库（2026-03 最新 action-v0.6.2）【MEDIUM-HIGH】  
- **Repo evidence**: `.github/workflows/e2e-tests.yml`, `e2e-tests/wdio.conf.mjs`, `e2e-tests/scripts/run-desktop-e2e.mjs`, `apps/client/web/app/playwright.config.ts`, `moon.yml`, `Justfile`, `.tool-versions`【HIGH】

---
*Stack research for: milestone v0.2.1 quality hardening*  
*Researched: 2026-04-06*
