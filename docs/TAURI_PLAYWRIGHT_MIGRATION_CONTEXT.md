# Tauri Playwright Migration Context

Last updated: 2026-04-07
Owner: E2E migration stream
Status: Ready for implementation

## 1) Purpose

This document is the handoff context for future agents to migrate desktop E2E from
`WDIO + tauri-driver` to `tauri-playwright` in a controlled, reversible way.

Primary objective:
- Add reliable macOS desktop E2E coverage without breaking existing Windows/Linux coverage.

Secondary objective:
- Gradually unify desktop and web test ergonomics around Playwright-style APIs.

## 2) Hard Constraints (Do Not Violate)

1. Do not change business logic, API contracts, or user-visible behavior.
2. Any test automation plugin must be gated behind `e2e-testing` feature or `debug_assertions`.
3. Release builds must not include desktop automation control surfaces.
4. Keep rollback path: old desktop E2E must remain runnable until migration is complete.
5. Minimize blast radius: migrate tests in phases, not big-bang.

## 3) Current Repository Baseline

Desktop E2E (current):
- `e2e-tests/wdio.conf.mjs`
- `e2e-tests/scripts/run-desktop-e2e.mjs`
- `e2e-tests/specs/*.e2e.mjs`

Web E2E (current):
- `apps/client/web/app/playwright.config.ts`
- `apps/client/web/app/tests/e2e/*.test.ts`

CI pipeline (current):
- `.github/workflows/e2e-tests.yml`
- Desktop matrix excludes macOS for WDIO/tauri-driver.

Native Tauri integration points:
- `apps/client/native/src-tauri/src/lib.rs`
- `apps/client/native/src-tauri/Cargo.toml`
- `apps/client/native/src-tauri/capabilities/default.json`
- `apps/client/native/src-tauri/tauri.conf.json`

Auth and tenant fixtures that must remain behavior-compatible:
- `apps/client/web/app/tests/fixtures/auth.ts`
- `apps/client/web/app/tests/fixtures/tenant.ts`
- `e2e-tests/helpers/tenant.mjs`

## 4) Target Migration Strategy

Use `srsholmes/tauri-playwright` as the first candidate for macOS-capable desktop E2E.

Reference:
- `https://github.com/srsholmes/tauri-playwright`

Adopt a dual-mode test strategy:
1. `browser` mode for fast feedback and broad regression.
2. `tauri` mode for real desktop webview validation.

Do not delete WDIO immediately. Keep it as fallback during transition.

## 5) Planned Phases

### Phase 0 - Bootstrap (no test migration yet)

Deliverables:
- Add feature-gated Rust plugin wiring for `tauri-plugin-playwright`.
- Add required capability permission (example in upstream README uses `playwright:default`).
- Add a new desktop Playwright config and fixture scaffold.
- Add one smoke test proving the stack works on local machine.

Exit criteria:
- `tauri` mode can connect and execute at least one assertion.

### Phase 1 - Minimum viable migration

Migrate first:
- smoke
- login
- counter (basic interactions only)

Keep old WDIO suite untouched and runnable.

Exit criteria:
- New tauri-playwright suite passes on macOS for migrated tests.
- Existing web Playwright remains green.

### Phase 2 - Core functional migration

Migrate:
- admin
- agent (including key interaction states)

Exit criteria:
- Equivalent assertions exist in new suite.
- Failure artifacts are useful for debugging.

### Phase 3 - Advanced behavior and decommission decision

Migrate last:
- tenant isolation
- any tests with cross-route + stateful setup complexity

Decision gate:
- If tauri-playwright stability and diagnostics are acceptable, deprecate WDIO.
- Otherwise keep hybrid model.

## 6) Definition of Done (per phase)

Each phase is complete only if all are true:
- Tests pass locally in intended mode(s).
- CI jobs are updated and passing for impacted matrix entries.
- Artifacts are available (report + screenshots, and video if configured).
- Rollback command/path is documented.
- No release build behavior changes.

## 7) Required Code Touchpoints Checklist

Future agents should check and update these intentionally:

1. Rust dependency and feature gating:
- `apps/client/native/src-tauri/Cargo.toml`

2. Plugin registration location:
- `apps/client/native/src-tauri/src/lib.rs`

3. Capability permission:
- `apps/client/native/src-tauri/capabilities/default.json`

4. Desktop E2E package/config (new):
- Suggested path: `e2e-desktop-playwright/` or reuse `apps/client/web/app` test harness with separate project config.

5. CI wiring:
- `.github/workflows/e2e-tests.yml`

## 8) Compatibility Rules for Test Behavior

When migrating any test, preserve these behaviors:

1. Auth flow assumptions:
- Existing tests rely on mock deep-link callback behavior.
- Equivalent mechanism must exist in migrated suite.

2. Route guarding assertions:
- Protected routes may validly resolve to either target page or login depending on auth state.
- Do not over-tighten assertions unless product behavior changed intentionally.

3. Tenant isolation setup:
- Keep tenant fixture identities stable (`tenant_a_user`, `tenant_b_user`) unless explicit migration plan updates both suites.

## 9) CI Design Guidance

Recommended transitional CI model:

1. Keep current jobs:
- desktop-e2e (WDIO, Windows/Linux)
- web-e2e (Playwright, Ubuntu/Windows/macOS)

2. Add new job:
- desktop-e2e-playwright-tauri (start with macOS only)

3. Promote gradually:
- If stable, expand to Windows/Linux and compare runtime + flake rate.

4. Decommission rule:
- Remove WDIO only after 2 consecutive weeks of stable tauri-playwright runs on all required platforms.

## 10) Quality Gates and Metrics

Track these explicitly in PR descriptions:
- Pass rate per platform
- Median runtime per suite
- Flake rate (rerun sensitivity)
- Mean time to diagnose failures

Minimum acceptance recommendation:
- Flake rate not worse than baseline by more than 5 percentage points.
- Runtime increase not worse than 20 percent unless diagnostics improve significantly.

## 11) Known Risks and Mitigations

Risk: test plugin accidentally leaks into production
- Mitigation: feature-gated dependency + conditional plugin registration + CI release build check.

Risk: fixture drift between old/new suites
- Mitigation: centralize shared fixture constants and compare behavior before deleting WDIO tests.

Risk: brittle selectors during migration
- Mitigation: prefer `data-testid` for new/ported tests.

Risk: hidden regressions due to early WDIO removal
- Mitigation: run both suites in parallel during migration window.

## 12) Rollback Plan

If migration introduces instability:
1. Disable new tauri-playwright CI job.
2. Keep WDIO pipeline as source of truth.
3. Revert only migration-layer commits; do not touch product code.
4. Re-open migration with narrowed scope (smoke only).

## 13) Agent Execution Contract

Any future agent implementing this migration must provide in each PR:
- What changed
- Why this phase is safe
- How it was verified
- Remaining risks
- Exact rollback steps

And must avoid:
- deleting legacy WDIO suite before decision gate
- changing application behavior to satisfy tests
- introducing production-exposed automation endpoints

## 14) Ready-to-use Prompt for Next Agent

```text
Task: Implement Phase 0 of desktop E2E migration to tauri-playwright.

Context file:
- docs/TAURI_PLAYWRIGHT_MIGRATION_CONTEXT.md

Hard constraints:
1) No business logic changes.
2) Test plugin only under e2e-testing feature/debug assertions.
3) Keep WDIO suite runnable.

Repository touchpoints:
- apps/client/native/src-tauri/Cargo.toml
- apps/client/native/src-tauri/src/lib.rs
- apps/client/native/src-tauri/capabilities/default.json
- .github/workflows/e2e-tests.yml

Deliverables:
- Feature-gated tauri-playwright wiring
- New desktop playwright tauri-mode scaffold
- One smoke test passing locally
- Short migration notes and rollback steps

Verification:
- Run only impacted tests/checks
- Report exact commands and outcomes
```
