---
phase: 05
slug: database-infrastructure
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-29
---

# Phase 05 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust workspace) |
| **Config file** | Cargo.toml workspace |
| **Quick run command** | `cargo check --workspace` |
| **Full suite command** | `cargo test --workspace` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo check --workspace`
- **After every plan wave:** Run `cargo test --workspace`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 60 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 05-01-01 | 01 | 1 | INFRA-01 | compile | `cargo check -p domain` | ✅ | ⬜ pending |
| 05-01-02 | 01 | 1 | INFRA-01 | compile | `cargo check -p runtime_server` | ✅ | ⬜ pending |
| 05-02-01 | 02 | 1 | INFRA-01 | compile | `cargo check -p runtime_server` | ✅ | ⬜ pending |
| 05-02-02 | 02 | 1 | INFRA-02 | compile | `cargo check -p runtime_server` | ✅ | ⬜ pending |
| 05-03-01 | 03 | 2 | INFRA-01 | compile | `cargo check -p runtime_server` | ✅ | ⬜ pending |
| 05-03-02 | 03 | 2 | INFRA-04 | compile | `cargo check -p desktop-ui-tauri` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- None — existing infrastructure covers all phase requirements. Cargo workspace is configured, `cargo check` works.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Axum server starts with DB pool | INFRA-01 | Requires running server | `cargo run -p runtime_server` then curl `/readyz` |
| Tauri app opens with libsql plugin | INFRA-01 | Requires Tauri runtime | `bun run tauri dev` check console for libsql init |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 60s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending

---

*Phase 05: Database & Infrastructure*
*2026-03-29*
