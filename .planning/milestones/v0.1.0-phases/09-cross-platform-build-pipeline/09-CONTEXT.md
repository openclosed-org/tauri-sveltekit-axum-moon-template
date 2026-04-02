# Phase 9: Cross-Platform Build Pipeline - Context

**Gathered:** 2026-03-30
**Status:** Ready for planning

<domain>
## Phase Boundary

Cross-platform build verification: ensure the project compiles and bundles on Windows, macOS, and Linux with CI enforcement. This phase covers CI matrix configuration, platform-specific installer defaults, and platform edge-case handling.

Out of scope: release automation, artifact publishing, and notarization/code-signing pipelines.

**Success Criteria (from ROADMAP.md):**
1. `cargo build` completes without errors on Windows (with WebView2)
2. `cargo build` completes without errors on macOS (with entitlements)
3. `cargo build` completes without errors on Linux
4. CI configuration exists to verify all three platform builds

</domain>

<decisions>
## Implementation Decisions

### CI Strategy
- **D-01:** Use GitHub Actions matrix with 3 jobs: `ubuntu-latest`, `windows-latest`, `macos-latest`
- **D-02:** Trigger CI on `push` to main and `pull_request` (replace manual-only trigger)
- **D-03:** Use Bun on all platforms for consistency with local workflow
- **D-04:** Each platform job runs: system deps setup (as needed) -> Rust toolchain -> bun install -> checks/tests -> build verification

### Release Automation Scope
- **D-05:** Phase 9 delivers build verification only
- **D-06:** No release workflow, no artifact upload, no changelog automation in this phase

### Installer & Bundling
- **D-07:** Use boilerplate-ready installer defaults (light customization, not full production packaging)
- **D-08:** Windows installer format: NSIS
- **D-09:** macOS uses basic entitlements with hardened runtime baseline (no notarization workflow)
- **D-10:** Keep `bundle.targets: "all"` in Tauri config

### Platform Edge Cases
- **D-11:** WebView2 handled by Tauri default bootstrapper strategy (no offline bundling)
- **D-12:** Linux dependencies installed in CI via apt packages (webkit2gtk, gtk3, appindicator, etc.)
- **D-13:** Use native compilation per platform runner (no cross-compilation)

### the agent's Discretion
- CI cache strategy details (Rust/Bun cache keys)
- Whether to verify with `cargo build`, `cargo tauri build`, or both in matrix jobs
- Exact NSIS option surface (license file path, install mode, optional UI tweaks)
- Exact macOS entitlements file contents within "basic" scope
- Whether to add helper moon tasks for CI wrappers

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase and requirements anchors
- `.planning/ROADMAP.md` — Phase 9 goal, dependency chain, and success criteria
- `.planning/REQUIREMENTS.md` — `BUILD-02` acceptance requirement

### Existing CI and build pipeline files
- `.github/workflows/ci.yml` — Current CI baseline (Ubuntu-only, manual trigger)
- `moon.yml` — Root build/check/lint/test task aggregation
- `apps/desktop-ui/moon.yml` — Desktop UI task definitions including Tauri build command

### Tauri build configuration
- `apps/desktop-ui/src-tauri/tauri.conf.json` — Bundle targets and app packaging config
- `apps/desktop-ui/src-tauri/Cargo.toml` — Tauri crate dependency surface

### Prior phase decisions that constrain this phase
- `.planning/phases/04-backend-dependencies-build-optimization/04-CONTEXT.md` — Release profile and binary optimization baseline
- `.planning/phases/08-desktop-native-features/08-CONTEXT.md` — Desktop runtime expectations and platform behavior assumptions

### External platform docs (implementation guidance)
- `https://v2.tauri.app/distribute/` — Tauri distribution overview
- `https://v2.tauri.app/reference/config/#bundleconfig` — Bundle config reference
- `https://v2.tauri.app/distribute/windows-installer/` — NSIS installer guidance
- `https://v2.tauri.app/distribute/macos-application/` — macOS entitlements/signing guidance
- `https://v2.tauri.app/distribute/linux/` — Linux packaging guidance

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `.github/workflows/ci.yml`: existing quality-gate workflow can be expanded to matrix instead of rebuilt from scratch
- `moon.yml`: shared Rust/frontend task orchestration already exists
- `apps/desktop-ui/moon.yml`: has `tauri:build` task pattern usable for CI task consistency
- `apps/desktop-ui/src-tauri/tauri.conf.json`: already includes multi-platform icon assets and `bundle.targets: "all"`

### Established Patterns
- CI currently uses `actions/checkout`, `dtolnay/rust-toolchain`, `Swatinem/rust-cache`, `oven-sh/setup-bun`
- Linux system libraries are installed directly in workflow steps
- Workspace checks often exclude `desktop-ui-tauri` for generic Rust quality gates

### Integration Points
- Update `.github/workflows/ci.yml` to matrix and trigger policy
- Potentially extend `apps/desktop-ui/src-tauri/tauri.conf.json` with NSIS/basic macOS entitlement references
- Optional: introduce CI-oriented helper tasks in `moon.yml` / `apps/desktop-ui/moon.yml`

</code_context>

<specifics>
## Specific Ideas

- Matrix strategy should keep per-OS setup isolated and explicit for easier troubleshooting
- Keep build verification and release orchestration separate to avoid phase scope creep
- NSIS is preferred as Windows default installer path for this boilerplate
- macOS stays at "basic entitlements" level to avoid mandatory secret/certificate setup in this phase
- Native platform runners are preferred over cross-compilation for Tauri reliability

</specifics>

<deferred>
## Deferred Ideas

- `release-plz` + `git-cliff` integration
- Tag-triggered GitHub release publishing workflow
- Artifact upload/download pipeline for CI runs
- macOS notarization automation (Apple account secrets)
- Windows code-signing pipeline (certificate management)
- Offline WebView2 bundling for air-gapped environments
- Dockerized Linux CI image strategy

</deferred>

---

*Phase: 09-cross-platform-build-pipeline*
*Context gathered: 2026-03-30*
