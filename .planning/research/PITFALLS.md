# Pitfalls Research

**Domain:** Cross-platform desktop release hardening (Tauri v2 + SvelteKit + Axum) for E2E + QA + bug-loop
**Researched:** 2026-04-06
**Confidence:** MEDIUM-HIGH

## Critical Pitfalls

### Pitfall 1: Flakiness masked by retries ("green but unstable")

**What goes wrong:**
CI goes green due to retries, but underlying race/timing defects remain. Release quality degrades silently.

**Why it happens:**
Teams add `retries: 2` quickly, but do not quarantine flaky tests or track first-run pass rate.

**How to avoid:**
- Keep retries only as containment, not as success criterion.
- Gate on **first-run pass rate** (e.g., >= 98%) and flaky-test budget.
- Capture trace/video/screenshot on retry/failure (`trace: on-first-retry`, `video: on-first-retry`, `screenshot: only-on-failure`).
- Require bug-loop ticket for any flaky test seen >N times/week.

**Warning signs / detection signals:**
- "Flaky" label count rising in Playwright report.
- Same spec alternates pass/fail across reruns.
- MTTR for test failures getting longer.

**Phase to address:**
Phase 1 (quality policy + baseline metrics) and Phase 3 (stabilization loop).

---

### Pitfall 2: Time-based waits and brittle selectors in E2E

**What goes wrong:**
Tests pass locally but fail on CI runners (especially Windows/macOS load variance).

**Why it happens:**
Using `waitForTimeout` and CSS/XPath selectors coupled to DOM structure.

**How to avoid:**
- Ban fixed sleeps in production tests.
- Standardize on Playwright locators (`getByRole`, `getByTestId`) + web-first assertions.
- Add lint/check that fails on `waitForTimeout` outside debug-tagged tests.

**Warning signs / detection signals:**
- Failures cluster around "element not found/visible" and timeout errors.
- Test duration variance spikes between runs.

**Phase to address:**
Phase 2 (test design standards + linting).

---

### Pitfall 3: Cross-test state leakage (accounts, DB rows, storageState)

**What goes wrong:**
Parallel workers contaminate each other (shared users, reused auth state, mutated fixtures), causing nondeterministic failures.

**Why it happens:**
Single shared test account and mutable global fixtures were fine at low parallelism, then collapse in CI matrix.

**How to avoid:**
- One account per worker for state-mutating tests (Playwright recommended pattern).
- Deterministic seed/reset strategy per suite; isolate tenant/test IDs.
- Split read-only vs state-mutating suites with different worker configs.

**Warning signs / detection signals:**
- Failures disappear when `workers=1`.
- Duplicate key / optimistic lock / "already exists" intermittently.
- Order-dependent failures.

**Phase to address:**
Phase 2 (data model for tests) and Phase 3 (parallel execution hardening).

---

### Pitfall 4: Environment drift across runners (windows-latest/macos-latest)

**What goes wrong:**
Previously stable pipeline breaks after hosted image/tooling updates; false "app regression" alarm.

**Why it happens:**
Using floating runner labels and implicit tool versions without changelog monitoring.

**How to avoid:**
- Pin OS images for release gates (`windows-2025`, explicit macOS label) and pin major tool versions.
- Add scheduled canary against `*-latest` to detect upcoming breakage early.
- Log runner image version in CI artifacts.

**Warning signs / detection signals:**
- Failures start same day with no app-code changes.
- Failures only in one OS lane.

**Phase to address:**
Phase 1 (CI topology), reinforced in Phase 4 (release gate hardening).

---

### Pitfall 5: Platform capability blind spots (WebView2 vs WKWebView)

**What goes wrong:**
Test strategy assumes same automation surface on Windows/macOS; one platform has reduced E2E coverage.

**Why it happens:**
Tauri desktop E2E tooling capability differs by platform; teams design a single-path harness then discover macOS gaps late.

**How to avoid:**
- Define per-platform E2E/UAT matrix explicitly (Windows lane, macOS lane, and fallback UAT protocol if tooling limits apply).
- Keep contract-level integration tests strong where UI automation is weaker.
- Track uncovered risk areas by platform in release checklist.

**Warning signs / detection signals:**
- "Passed on Windows, unverified on macOS" before release.
- Manual UAT repeatedly finds issues not seen in automation.

**Phase to address:**
Phase 1 (test strategy) and Phase 4 (release UAT protocol).

---

### Pitfall 6: Happy-path-only suites create false confidence

**What goes wrong:**
Smoke tests pass, but real release fails under auth expiry, offline/retry, reconnect, malformed payloads, tenant boundary edges.

**Why it happens:**
E2E scope optimized for demo paths; no risk-based negative-path coverage.

**How to avoid:**
- Define release-gate "failure-mode pack" (auth refresh fail, network drop/reconnect, stale cache, duplicate submit, 4xx/5xx handling).
- Map each known tech debt area (AUTH-01, offline sync gap) to at least one QA/UAT scenario.
- Add bug-loop template that requires reproduction in negative-path class.

**Warning signs / detection signals:**
- Incidents come from expired sessions/offline states not represented in test reports.
- High production bug ratio in edge conditions.

**Phase to address:**
Phase 2 (coverage model), Phase 3 (bug-loop), Phase 4 (release gate criteria).

---

### Pitfall 7: Missing observability in bug-loop (no forensic artifacts)

**What goes wrong:**
Failures cannot be triaged quickly because no trace, no app logs, no correlation ID, no reproducible env snapshot.

**Why it happens:**
Pipeline optimized for pass/fail speed, not debug recoverability.

**How to avoid:**
- Mandatory artifact bundle on failure: Playwright trace + screenshot/video + app logs + test metadata.
- Correlation IDs from UI action → Tauri command → Axum request.
- Standard "repro packet" format attached to bug ticket.

**Warning signs / detection signals:**
- "Cannot reproduce" loops exceed 2 handoffs.
- Same failure reopened multiple times due to insufficient evidence.

**Phase to address:**
Phase 3 (bug-loop instrumentation).

---

### Pitfall 8: Release gate blind spots in GitHub workflows

**What goes wrong:**
Code merges/release proceeds without actually running all required quality checks (event mismatch, skipped workflows, missing merge queue trigger).

**Why it happens:**
Branch/path filters + merge queue not reflected in workflow triggers; required checks misconfigured.

**How to avoid:**
- Ensure required workflows trigger on `pull_request` **and** `merge_group` when merge queue is enabled.
- Keep required checks minimal-but-sufficient and stable in name.
- Audit branch protection quarterly.

**Warning signs / detection signals:**
- PR shows pending/skipped checks unexpectedly.
- Post-merge failures for checks that did not run pre-merge.

**Phase to address:**
Phase 4 (release governance and gate enforcement).

---

### Pitfall 9: Unsigned/partially signed build tested as if release-equivalent

**What goes wrong:**
UAT passes on unsigned dev builds, but release candidate fails install/launch/security prompts on macOS/Windows.

**Why it happens:**
Signing/notarization happens too late, outside QA/UAT loop.

**How to avoid:**
- Introduce "release-like" signed artifacts early in RC cycle.
- Include install/upgrade/uninstall and trust-prompt checks in UAT script.
- Separate quick CI smoke from release-candidate distribution validation.

**Warning signs / detection signals:**
- QA validates unpacked binaries only.
- Last-minute signing pipeline failures block release.

**Phase to address:**
Phase 4 (release candidate validation).

## Technical Debt Patterns (Do-Not-Do Shortcuts)

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|----------------|-----------------|
| "Just increase retries" | Temporary green pipeline | Hides instability; confidence collapse | Never as sole fix |
| `waitForTimeout` everywhere | Easy to implement | Chronic flakiness across CI/OS | Debug-only, not committed |
| Single shared QA account | Fast setup | Parallel collisions, nondeterminism | Only for read-only smoke |
| Floating `*-latest` everywhere | Less YAML maintenance | Sudden runner drift incidents | Never for release-gate lanes |
| Happy-path-only E2E | Fast execution | Major blind spots in failure modes | Only pre-milestone spike |
| Skip artifact retention for failures | Lower storage cost | Slow triage, repeated bug loops | Never for release branches |
| Manual pass/fail in chat without evidence | Fast sign-off | Surface-pass outcomes, audit gaps | Never |

## Integration Gotchas (Windows/macOS + Tauri/Axum)

| Integration | Common Mistake | Correct Approach |
|-------------|----------------|------------------|
| Tauri desktop E2E | Assume identical WebDriver capability on all desktop OS | Design per-platform strategy and explicit coverage gaps; compensate with integration + UAT scripts |
| Tauri IPC + Axum backend | Validate only IPC path or only HTTP path | Keep dual-path regression pack (desktop IPC + browser/server path) |
| GitHub Actions matrix | One matrix, no OS-specific dependencies/checks | Separate Windows/macOS job setup and platform diagnostics |
| Signing/notarization | Run only unsigned app in QA | Validate signed RC artifacts before gate decision |
| Playwright artifacts | Upload everything blindly (including secrets) | Redact/secure artifacts; trusted storage only |

## "Looks Done But Isn't" Checklist

- [ ] **Flake control:** Have we tracked first-run pass rate (not just final pass after retries)?
- [ ] **Data isolation:** Can all state-mutating tests run in parallel without shared account collisions?
- [ ] **Cross-platform parity:** Are Windows and macOS both represented in release gate with explicit pass criteria?
- [ ] **Negative paths:** Are offline/retry/auth-expiry/error-recovery flows in gate suite, not only smoke suite?
- [ ] **Bug-loop evidence:** Does every failed gate test produce trace + logs + repro metadata automatically?
- [ ] **Workflow integrity:** Do required checks run for PR and merge queue paths (`merge_group`), not just direct PR?
- [ ] **Release realism:** Was UAT executed against signed/notarized release-like artifact?

## Pitfall-to-Phase Mapping

| Pitfall | Prevention Phase | Verification |
|---------|------------------|--------------|
| Flakiness masked by retries | Phase 1 + 3 | Dashboard shows first-run pass rate, flaky budget, quarantine SLA |
| Time-based waits & brittle selectors | Phase 2 | Lint rule + grep check: no committed hard waits |
| State leakage in parallel tests | Phase 2 + 3 | `workers>1` stable for N consecutive runs |
| Runner environment drift | Phase 1 + 4 | Pinned release lanes + canary latest lane alerting |
| Platform capability blind spots | Phase 1 + 4 | Platform-specific coverage map signed off at gate |
| Happy-path-only false confidence | Phase 2 + 4 | Failure-mode pack present in gate suite |
| Weak bug-loop observability | Phase 3 | Failure artifact bundle auto-generated + linked ticket |
| Workflow release-gate blind spots | Phase 4 | Branch protection audit + merge_queue dry run |
| Unsigned build used for UAT | Phase 4 | UAT checklist requires signed RC artifact ID |

## Sources

1. Playwright Best Practices (locators, isolation, web-first assertions, CI guidance) — https://playwright.dev/docs/best-practices **[HIGH]**
2. Playwright CI intro (artifacts, trace workflow, secret handling in reports) — https://playwright.dev/docs/ci-intro **[HIGH]**
3. Playwright API/docs via Context7 (`retries`, `trace`, `video`, auth worker fixtures, `waitForTimeout` discouraged) — /microsoft/playwright.dev **[HIGH]**
4. Tauri v2 testing docs via Context7 (WebDriver testing, platform caveats incl. macOS desktop WebDriver client limitation note) — /websites/v2_tauri_app **[MEDIUM]**
5. Tauri v2 signing/distribution docs via Context7 (macOS signing requirements, CI examples) — /websites/v2_tauri_app **[HIGH]**
6. GitHub Actions workflow syntax/docs via Context7 (`merge_group`, skipped workflow pending checks, matrix strategy) — /websites/github_en_actions **[HIGH]**
7. GitHub Actions runner-images README (weekly updates, `-latest` migration behavior, pinning rationale) — https://github.com/actions/runner-images **[HIGH]**

## Gaps / Uncertainty

- Exa semantic search hit rate limits during this run; external incident-postmortem coverage is incomplete. Current recommendations prioritize official docs + platform documentation.
- macOS desktop WebDriver tooling ecosystem is evolving; verify current Tauri macOS E2E tooling status during phase execution before finalizing automation depth.

---
*Pitfalls research for: milestone v0.2.1 quality hardening (E2E + QA + bug-loop)*
*Researched: 2026-04-06*
