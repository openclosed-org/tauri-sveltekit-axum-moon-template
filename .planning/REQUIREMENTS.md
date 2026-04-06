# Requirements: Tauri-SvelteKit-Axum Boilerplate

**Defined:** 2026-04-06
**Milestone:** v0.2.1
**Core Value:** Provide a runnable, tested, production-ready engineering base with Google Auth, Counter, Admin Web, Agent conversation, contracts/typegen single-truth-source, and clear architectural boundaries.

## v0.2.1 Requirements

### Release Gate & Evidence

- [ ] **QGATE-01**: Maintainer can merge to protected branches only when Windows desktop E2E required check passes.
- [ ] **QGATE-02**: Maintainer can verify release readiness from Windows and macOS QA/UAT evidence for the same candidate build.
- [ ] **QGATE-03**: Maintainer can view a release quality summary that includes automated test results, UAT sign-offs, and open defects by severity.

### Defect Lifecycle Governance

- [ ] **BUG-01**: Maintainer can triage each bug with a standard severity (P0-P3), workflow state, and owner.
- [ ] **BUG-02**: Maintainer can close P0/P1 bugs only when linked regression verification evidence exists.
- [ ] **BUG-03**: Maintainer can track flaky tests in a dedicated quarantine flow with repair SLA and status visibility.

### Authentication & Session

- [ ] **AUTH-02**: Signed-in user can click a visible Google logout action to sign out.
- [ ] **AUTH-03**: Signed-in user returns to unauthenticated state after logout with session credentials cleared across desktop and browser paths.

### Multi-tenant Verification

- [ ] **MTEN-01**: Tester can switch between at least two tenants in a repeatable test harness.
- [ ] **MTEN-02**: Tester can verify counter values are tenant-scoped, where tenant-1 changes do not alter tenant-2 values.
- [ ] **MTEN-03**: Maintainer can run automated multi-tenant tests in CI and collect artifacts for diagnosis.

### Functional Bug Fixes

- [ ] **COUNTER-02**: User can increment and decrement the counter and observe correct value changes in UI and persisted state.
- [ ] **AGENT-02**: User can click New Chat and start a new conversation thread.
- [ ] **AGENT-03**: User can click New Chat without resetting saved API key, base URL, and model settings.
- [ ] **AGENT-04**: User can click a connectivity-test action to validate API key, base URL, and model reachability with actionable result feedback.

## v0.2.x Requirements (Deferred)

### Quality Hardening Enhancements

- **QGATE-04**: Maintainer can enforce merge queue checks for all protected branches with periodic required-check audit.
- **REG-01**: Maintainer can run risk-based selective regression from changed paths with automatic fallback to full-suite on uncertainty.
- **MTEN-04**: Tester can run multi-tenant stress scenarios (parallel tenant mutation + recovery) to detect cross-tenant leakage under load.

## Out of Scope (This Milestone)

| Feature | Reason |
|---------|--------|
| macOS desktop WebDriver parity with Windows | Current ecosystem support is not stable enough for v0.2.1 release gate commitments |
| Unlimited retries to force green CI | Masks regressions and reduces release-signal trust |
| Full replacement of existing test stack (WDIO/Playwright/moon/Just) | Violates brownfield minimal-change strategy and increases migration risk |
| Blocking release on all P2/P3 defects | Would reduce delivery throughput without proportional risk reduction |

## Traceability

Which phases cover which requirements. Updated during roadmap creation.

| Requirement | Phase | Status |
|-------------|-------|--------|
| QGATE-01 | TBD | Pending |
| QGATE-02 | TBD | Pending |
| QGATE-03 | TBD | Pending |
| BUG-01 | TBD | Pending |
| BUG-02 | TBD | Pending |
| BUG-03 | TBD | Pending |
| AUTH-02 | TBD | Pending |
| AUTH-03 | TBD | Pending |
| MTEN-01 | TBD | Pending |
| MTEN-02 | TBD | Pending |
| MTEN-03 | TBD | Pending |
| COUNTER-02 | TBD | Pending |
| AGENT-02 | TBD | Pending |
| AGENT-03 | TBD | Pending |
| AGENT-04 | TBD | Pending |

**Coverage:**
- v0.2.1 requirements: 15 total
- Mapped to phases: 0
- Unmapped: 15 ⚠️

---

*Requirements defined: 2026-04-06*
*Last updated: 2026-04-06 after milestone v0.2.1 scoping and research*
