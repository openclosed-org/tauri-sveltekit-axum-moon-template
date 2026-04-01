# Requirements: Tauri-SvelteKit-Axum Boilerplate

**Defined:** 2026-04-01
**Milestone:** v0.1.1
**Core Value:** Provide a runnable, tested, production-ready boilerplate with authentication, multi-tenancy, and full-stack best practices so developers can start business implementation immediately.

## v0.1.1 Requirements

### Security Baseline

- [ ] **SEC-01**: Server rejects invalid or forged JWT for tenant/auth-critical entry points
- [ ] **SEC-02**: App/server fail fast when required secrets are missing in non-dev environments
- [ ] **SEC-03**: Runtime config and path resolution are platform-portable with no hardcoded machine-specific absolute paths

### Contracts & Type Sync

- [ ] **CONTRACT-01**: `contracts_api` defines shared DTO/contracts as the single Rust source of truth
- [ ] **CONTRACT-02**: `typegen` generates TS types from Rust contracts and is enforced by verify/CI drift checks

### Runtime Boundary Convergence

- [ ] **RUNTIME-01**: New orchestration logic is owned by `runtime_tauri` and `apps/client/native/src-tauri` remains host/bootstrap focused

### Workflow Guardrails

- [ ] **WF-01**: Unified developer task entrypoints exist for `fullstack:dev`, `typegen`, and `verify` across Moon/Just

### Decision Ledger & Forward Map

- [ ] **DECISION-01**: All strategy items from milestone discussion are recorded with status (`implement-now` / `defer` / `reject`) and rationale
- [ ] **DECISION-02**: Deferred and rejected items include concise future-phase mapping and promotion triggers

## Future Requirements (Deferred)

### Security / Governance

- **DF-01**: Implement full JWKS cache and key-rotation strategy
- **DF-02**: Introduce fine-grained RBAC model

### Tooling / Contracts

- **DF-03**: Automate decision-ledger generation from planning artifacts
- **DF-04**: Introduce cross-language contract version negotiation matrix

## Out of Scope (This Milestone)

| Feature | Reason |
|---------|--------|
| New auth mode (email/password) | Not needed for v0.1.1 convergence goals and expands scope/risk |
| New business modules/pages | v0.1.1 is architecture closure, not feature expansion |
| Full stack or framework replacement | Violates minimum-change principle and increases regression risk |
| New heavyweight infrastructure services | Low ROI for this milestone's closure goals |
| `tauri-plugin-axum` as primary path now | Deferred to later evaluation after contracts and boundary closure |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| SEC-01 | TBD | Pending |
| SEC-02 | TBD | Pending |
| SEC-03 | TBD | Pending |
| CONTRACT-01 | TBD | Pending |
| CONTRACT-02 | TBD | Pending |
| RUNTIME-01 | TBD | Pending |
| WF-01 | TBD | Pending |
| DECISION-01 | TBD | Pending |
| DECISION-02 | TBD | Pending |

**Coverage:**
- v0.1.1 requirements: 9 total
- Mapped to phases: 0
- Unmapped: 9 ⚠️

---
*Requirements defined: 2026-04-01*
*Last updated: 2026-04-01 after milestone v0.1.1 requirement confirmation*
