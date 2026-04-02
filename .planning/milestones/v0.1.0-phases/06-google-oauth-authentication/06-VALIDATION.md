---
phase: 06
slug: google-oauth-authentication
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-29
---

# Phase 06 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Vitest (frontend), cargo test (Rust) |
| **Config file** | `apps/desktop-ui/vitest.config.ts` (frontend), workspace Cargo.toml (Rust) |
| **Quick run command** | `cd apps/desktop-ui && npx vitest run src/lib/ipc/auth.test.ts` |
| **Full suite command** | `cd apps/desktop-ui && npx vitest run && cargo test` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `npx vitest run` for affected test file
- **After every plan wave:** Run `npx vitest run && cargo test --lib`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 06-01-01 | 01 | 1 | AUTH-02 | unit (Rust) | `cargo test auth` | ❌ W0 | ⬜ pending |
| 06-01-02 | 01 | 1 | AUTH-02 | unit (Rust) | `cargo test deep_link` | ❌ W0 | ⬜ pending |
| 06-02-01 | 02 | 1 | AUTH-01 | unit (Rust) | `cargo test oauth` | ❌ W0 | ⬜ pending |
| 06-02-02 | 02 | 1 | AUTH-03 | unit (Rust) | `cargo test session_store` | ❌ W0 | ⬜ pending |
| 06-03-01 | 03 | 2 | AUTH-04 | unit (Rust) | `cargo test refresh` | ❌ W0 | ⬜ pending |
| 06-04-01 | 04 | 2 | AUTH-01, AUTH-03 | component (Vitest) | `npx vitest run login` | ❌ W0 | ⬜ pending |
| 06-04-02 | 04 | 2 | AUTH-03 | component (Vitest) | `npx vitest run auth-store` | ❌ W0 | ⬜ pending |
| 06-05-01 | 05 | 3 | AUTH-01-AUTH-04 | integration | `npx vitest run && cargo test` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `apps/desktop-ui/src/lib/ipc/auth.test.ts` — IPC wrapper test stubs
- [ ] `apps/desktop-ui/src/lib/stores/auth.test.ts` — Auth store test stubs
- [ ] `crates/runtime_tauri/src/commands/auth.rs` — Auth command stubs with test modules

*Executor must create test stubs before implementing features.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Deep link callback capture | AUTH-02 | Requires running Tauri app + real Google OAuth | 1. `bun run tauri dev` 2. Click Google login 3. Authorize in browser 4. Verify callback captured |
| Login → Counter redirect | AUTH-01 | UI flow requires running app | 1. Complete login 2. Verify automatic redirect to /counter |
| Session restore after restart | AUTH-03 | Requires app restart | 1. Login 2. Close app 3. Reopen 4. Verify still logged in |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
