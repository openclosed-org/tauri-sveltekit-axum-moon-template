# Phase 9: Cross-Platform Build Pipeline - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md. This log preserves options considered and user selections.

**Date:** 2026-03-30
**Phase:** 09-cross-platform-build-pipeline
**Areas discussed:** CI strategy, release automation, installer & bundling scope, platform edge cases

---

## CI Strategy

### Cross-platform verification model

| Option | Description | Selected |
|--------|-------------|----------|
| GitHub Actions matrix | 3-job matrix (Linux/Windows/macOS), CI as source of truth | ✓ |
| Moon tasks only | Local-only verification, no CI enforcement | |
| Both moon + CI matrix | Local convenience + CI enforcement | |

**User's choice:** GitHub Actions matrix
**Notes:** Cross-platform verification should be enforced by CI, not left to manual local runs.

### CI trigger policy

| Option | Description | Selected |
|--------|-------------|----------|
| Push + PR | Trigger on push to main and pull requests | ✓ |
| PR only | Trigger only on PR events | |
| Manual only | Keep current workflow_dispatch only mode | |

**User's choice:** Push + PR
**Notes:** Current manual-only trigger should be replaced.

### Frontend package manager in CI

| Option | Description | Selected |
|--------|-------------|----------|
| Bun all platforms | Bun on Linux/macOS/Windows for consistency | ✓ |
| Bun + npm split | Bun on Linux/macOS, npm on Windows | |

**User's choice:** Bun all platforms
**Notes:** Keep CI toolchain aligned with project local workflow.

---

## Release Automation

### Phase 9 release scope

| Option | Description | Selected |
|--------|-------------|----------|
| Build verification only | Only deliver cross-platform build verification in this phase | ✓ |
| Build + release workflow | Add tag-triggered release job and artifact publish | |
| Full pipeline | Add release-plz + git-cliff + release automation in this phase | |

**User's choice:** Build verification only
**Notes:** Keep strict phase boundary; release automation is deferred.

### CI artifact upload

| Option | Description | Selected |
|--------|-------------|----------|
| Upload artifacts | Upload installers/binaries from CI runs | |
| Compile check only | No artifact upload in this phase | ✓ |

**User's choice:** Compile check only
**Notes:** Artifacts should be tied to a future release workflow phase.

---

## Installer & Bundling Scope

### Build vs installer customization level

| Option | Description | Selected |
|--------|-------------|----------|
| Build verification only | Only ensure compilation succeeds | |
| Build + default bundlers | Use Tauri defaults only | |
| Build + custom installers | Include installer customization in this phase | ✓ |

**User's choice:** Build + custom installers
**Notes:** Customization is required, but with controlled complexity.

### Bundle target strategy

| Option | Description | Selected |
|--------|-------------|----------|
| Keep `bundle.targets = "all"` | Let each platform build native bundle targets | ✓ |
| Explicit per-platform targets | Hardcode targets per OS | |

**User's choice:** Keep `bundle.targets = "all"`
**Notes:** Keep config simple and broadly reusable for boilerplate consumers.

### Customization depth

| Option | Description | Selected |
|--------|-------------|----------|
| Boilerplate-ready defaults | Sensible defaults without heavy production-only complexity | ✓ |
| Production-grade full customization | Deep installer/signing pipeline setup | |

**User's choice:** Boilerplate-ready defaults
**Notes:** Phase should provide practical defaults, not full enterprise distribution pipeline.

### Windows installer format

| Option | Description | Selected |
|--------|-------------|----------|
| NSIS | Standard Tauri Windows installer path | ✓ |
| MSI | Enterprise-oriented Windows Installer format | |
| NSIS + MSI | Maintain both installer formats | |

**User's choice:** NSIS
**Notes:** NSIS chosen as default boilerplate path.

### macOS entitlements level

| Option | Description | Selected |
|--------|-------------|----------|
| Basic entitlements | Hardened runtime baseline without notarization workflow | ✓ |
| Notarization-ready | Full signing + notarization secrets/workflow | |

**User's choice:** Basic entitlements
**Notes:** Avoid Apple account secret requirements in this phase.

---

## Platform Edge Cases

### Windows WebView2 strategy

| Option | Description | Selected |
|--------|-------------|----------|
| Tauri default (bootstrapper) | Use default WebView2 runtime bootstrap behavior | ✓ |
| Offline WebView2 bundle | Ship full runtime with installer | |
| Document-only | Require manual runtime install | |

**User's choice:** Tauri default (bootstrapper)
**Notes:** Keep installer size reasonable and use standard Tauri behavior.

### Linux CI system dependencies

| Option | Description | Selected |
|--------|-------------|----------|
| apt-get in CI | Install required GTK/WebKit deps in workflow | ✓ |
| Docker container | Use prebuilt image with dependencies | |
| apt-get + extra distro docs | Install in CI plus non-Debian docs | |

**User's choice:** apt-get in CI
**Notes:** Continue existing CI pattern.

### Native vs cross-compilation

| Option | Description | Selected |
|--------|-------------|----------|
| Native per platform | Build on each OS runner natively | ✓ |
| Cross-compile | Build all targets from one runner | |

**User's choice:** Native per platform
**Notes:** Prioritize Tauri compatibility and reliability.

---

## the agent's Discretion

No explicit "you decide" directives were given. Discretion remains only on implementation detail level (cache keys, exact task ordering, config field granularity) within locked decisions.

## Deferred Ideas

- release-plz + git-cliff integration
- Tag-triggered release workflow and artifact publishing
- macOS notarization automation
- Windows code-signing pipeline
- Offline WebView2 bundling
- Dockerized Linux CI image strategy
