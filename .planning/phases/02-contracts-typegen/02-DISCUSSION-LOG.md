# Phase 02: Contracts/typegen 单一真理源 - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-01
**Phase:** 02-contracts-typegen
**Areas discussed:** Typegen toolchain, Contract DTO scope, Output & integration, Drift detection CI

---

## Typegen toolchain

| Option | Description | Selected |
|--------|-------------|----------|
| ts-rs | Most mature, simple derive macro, works with serde. Good for pure DTO sharing. | ✓ |
| specta | Tauri-native, supports function signatures, but heavier and more opinionated. | |
| tsify | Minimal, lightweight. Less active development, fewer features. | |

**User's choice:** ts-rs (Recommended)
**Notes:** None

---

## Contract DTO scope

| Option | Description | Selected |
|--------|-------------|----------|
| Minimal | Move HealthResponse, InitTenantRequest/Response only. Prove the pipeline, iterate later. | |
| Phase 4 aware | Also define Counter/Agent DTOs. More upfront work but Phase 4 is pre-typed. | |
| Blueprint-aligned | Full contracts/api + auth + events. Matches blueprint structure but speculative. | ✓ |

**User's choice:** Blueprint-aligned
**Notes:** Wants contracts split by concern — api/ for route DTOs, auth/ for auth types, events/ for domain events.

### Contracts split follow-up

| Option | Description | Selected |
|--------|-------------|----------|
| By concern | api = route DTOs, auth = auth token/session types, events = domain event payloads. | ✓ |
| All in api/ | Everything in contracts/api. Simpler import paths, one crate to manage. | |

**User's choice:** By concern (Recommended)
**Notes:** None

---

## Output & integration

| Option | Description | Selected |
|--------|-------------|----------|
| $lib/generated/ | Standard SvelteKit convention. Easy imports via $lib/generated/api, etc. | |
| packages/contracts/generated/ | Co-located with contracts in packages/contracts/generated/. | ✓ |
| generated/ at root | At repo root, all consumers import from same path. | |

**User's choice:** packages/contracts/generated/
**Notes:** None

### Server usage follow-up

| Option | Description | Selected |
|--------|-------------|----------|
| Import from contracts | Server routes import from contracts_api crate. DTOs move from inline. | ✓ |
| Server keeps own DTOs | Server keeps its own DTOs. Only frontend uses generated types. | |

**User's choice:** Import from contracts (Recommended)
**Notes:** None

### Frontend imports follow-up

| Option | Description | Selected |
|--------|-------------|----------|
| $lib/generated alias | SvelteKit path alias $lib/generated → symlink or copy step. | ✓ |
| Direct relative path | Import directly from packages/contracts/generated/ using relative path. | |
| Dual output | Typegen outputs to BOTH packages/contracts/generated/ AND frontend src/lib/generated/. | |

**User's choice:** $lib/generated alias (Recommended)
**Notes:** None

---

## Drift detection CI

| Option | Description | Selected |
|--------|-------------|----------|
| Git diff after typegen | Run typegen, then `git diff --exit-code` on generated files. | |
| Checksum-based | Separate checksum/cache of generated output. | |
| Both | Both git diff + checksum for belt-and-suspenders. | ✓ |

**User's choice:** Both
**Notes:** None

### Verify hook follow-up

| Option | Description | Selected |
|--------|-------------|----------|
| Part of repo:verify | repo:contracts-check runs in repo:verify. All verify runs check drift. | |
| Standalone only | Separate task, called explicitly. Verify stays fast. | |
| Both | Both — runs in verify AND callable standalone. | ✓ |

**User's choice:** Both
**Notes:** None

---

## Agent's Discretion

No areas explicitly deferred to agent discretion during discussion.

## Deferred Ideas

None — discussion stayed within phase scope.
