---
phase: 07
slug: multi-tenant-data-isolation
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-29
---

# Phase 07 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test + rstest |
| **Config file** | none — crate-level tests |
| **Quick run command** | `cargo test -p runtime_server` |
| **Full suite command** | `cargo test --workspace` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p runtime_server`
- **After every plan wave:** Run `cargo test --workspace`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 07-01-01 | 01 | 1 | TENANT-01 | integration | `cargo test -p runtime_server -- tenant_schema` | ❌ W0 | ⬜ pending |
| 07-02-01 | 02 | 1 | TENANT-02 | unit | `cargo test -p runtime_server -- tenant_middleware` | ❌ W0 | ⬜ pending |
| 07-02-02 | 02 | 1 | TENANT-02 | unit | `cargo test -p runtime_server -- tenant_aware_db` | ❌ W0 | ⬜ pending |
| 07-03-01 | 03 | 2 | TENANT-03 | integration | `cargo test -p runtime_server -- ensure_tenant` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `crates/runtime_server/src/middleware/tenant.rs` — tenant extraction middleware + unit tests
- [ ] `crates/runtime_server/src/ports/surreal_db.rs` — TenantAwareSurrealDb 实现 + 注入测试
- [ ] `crates/runtime_server/src/routes/tenant.rs` — ensure_tenant API + integration test
- [ ] `crates/domain/src/ports/mod.rs` — TenantId newtype 定义

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Cross-tenant query returns empty (not error) | TENANT-02 | 需要两个不同 tenant 的数据 + 实际 SurrealDB 运行 | 1. 创建两个 tenant 的数据 2. 用 tenant A 的 token 查询 3. 确认 tenant B 的数据不可见 |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
