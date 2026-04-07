---
phase: 09
slug: functional-correctness-baseline-fix
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-06
---

# Phase 09 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | vitest + playwright |
| **Config file** | `apps/client/web/app/vitest.config.ts`, `apps/client/web/app/playwright.config.ts` |
| **Quick run command** | `bun run --cwd apps/client/web/app test:unit -- tests/component/auth.test.ts` |
| **Full suite command** | `bun run --cwd apps/client/web/app test:unit && bun run --cwd apps/client/web/app test:e2e --grep "(login|counter|agent)"` |
| **Estimated runtime** | ~90 seconds |

---

## Sampling Rate

- **After every task commit:** Run `bun run --cwd apps/client/web/app test:unit -- tests/component/auth.test.ts`
- **After every plan wave:** Run `bun run --cwd apps/client/web/app test:unit && bun run --cwd apps/client/web/app test:e2e --grep "(login|counter|agent)"`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 90 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 09-01-01 | 01 | 1 | AUTH-02, AUTH-03 | T-09-01 | Logout invalidates server/local session | unit+e2e | `bun run --cwd apps/client/web/app test:unit -- tests/component/auth.test.ts` | ✅ | ⬜ pending |
| 09-01-02 | 01 | 1 | AGENT-04 | T-09-02 | Connection test never leaks secrets and shows actionable result | unit | `bun run --cwd apps/client/web/app test:unit -- tests/component/settings-connection.test.ts` | ❌ W0 | ⬜ pending |
| 09-02-01 | 02 | 1 | COUNTER-02 | T-09-03 | Counter mutation errors are visible and non-destructive | unit+e2e | `bun run --cwd apps/client/web/app test:unit -- tests/component/counter.test.ts` | ✅ | ⬜ pending |
| 09-03-01 | 03 | 1 | AGENT-02, AGENT-03 | T-09-04 | New chat starts fresh thread and keeps config keys | unit+e2e | `bun run --cwd apps/client/web/app test:unit -- tests/component/agent-phase9.test.ts` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `apps/client/web/app/tests/component/settings-connection.test.ts` — stubs for AGENT-04
- [ ] `apps/client/web/app/tests/component/agent-phase9.test.ts` — stubs for AGENT-02/AGENT-03

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Settings 中 Logout 后返回公开页策略（优先上一公开页） | AUTH-03 | 需要浏览器历史栈验证 | 先访问公开页→登录→进入 settings→logout，验证返回策略 |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 90s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
