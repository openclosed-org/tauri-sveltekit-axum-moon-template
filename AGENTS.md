# AGENTS.md

> Thin coordination protocol for this repository. All agents read this first.
> Keep this file stable, short, and cross-cutting. Put volatile plans in `docs/_local/`.

## 1. Language Preference

Communication **MUST** be Chinese.

Code, commands, config keys, logs, and protocol fields stay in their original language.

## 2. Project Snapshot

This repository is an agent-first distributed backend harness template, not a generic app demo.

Default backend anchor: `counter-service`.

Business reference chain: `service -> contracts -> server -> outbox -> relay -> projector`.

Engineering reference chain: `platform model -> secrets -> deploy -> GitOps -> runbook`.

`apps/**` and `packages/ui/**` are optional shell surfaces. Root backend-core commands must not depend on frontend, desktop, mobile shells, or UI packages by default.

The harness is a coordination layer. It routes work, points to truth sources, and recommends gates. It does not prove semantic correctness by metadata alone.

## 3. Engineering Posture

Act as a senior distributed-systems engineer working in a long-lived codebase.

Default behavior:

1. Understand the invariant before changing code.
2. Fix the smallest causal closed loop, not the nearest symptom.
3. Prefer simple, explicit, typed, testable code over clever abstractions.
4. Preserve domain boundaries even when a shortcut would pass a gate faster.
5. Treat passing gates as evidence, not as the goal.
6. Leave the system easier to reason about than you found it.

Do not optimize for looking done. Optimize for restored behavior, durable correctness, and future maintainability.

## 4. Task Reading Paths

Backend tasks:

1. `AGENTS.md`
2. `docs/architecture/harness-philosophy.md`
3. `agent/codemap.yml`
4. `agent/manifests/routing-rules.yml`
5. `agent/manifests/gate-matrix.yml`

Tooling, scripts, gates, or repo-control tasks also read:

1. `justfile`
2. `justfiles/**`
3. `moon.yml`
4. `tools/repo-tools/**`
5. referenced executable helpers under `tools/repo-tools/**`, `infra/**/scripts/**`, or `ops/**/scripts/**`

Documentation or audit tasks also read `docs/README.md`.

Infra, secrets, topology, or deploy tasks also read:

1. `docs/operations/**`
2. `platform/model/**`
3. relevant `infra/**` and `ops/**`

Bug reports, failing gates, or regressions start from executable evidence: failing command, test, validator, schema, log, or reproduction path.

## 5. Truth Hierarchy

When determining current state, gather evidence in this order:

1. code, schemas, validators, tests, gates, scripts, and command output
2. generated artifacts only when produced from current sources and checked for drift
3. `platform/model/**` and `services/*/model.yaml`
4. `agent/**` manifests
5. prose documentation

Hard rules:

1. When docs or YAML conflict with executable sources, trust executable sources.
2. Never infer a file or module exists solely from target-state documentation.
3. YAML declarations can describe intent; they do not prove semantic correctness.
4. `declared`, `checked`, `tested`, and `proven` claims must cite executable evidence when raised above `declared`.

## 6. Routing

Full mapping lives in `agent/manifests/routing-rules.yml`.

Quick reference:

| Path                                                            | Primary owner      |
| --------------------------------------------------------------- | ------------------ |
| `platform/model/**`, `platform/schema/**`, `infra/**`, `ops/**` | platform-ops-agent |
| `packages/contracts/**`, `docs/contracts/**`                    | contract-agent     |
| `services/**`                                                   | service-agent      |
| `servers/**`                                                    | server-agent       |
| `workers/**`                                                    | worker-agent       |
| `apps/**`, `packages/ui/**`                                     | app-shell-agent    |
| `tools/repo-tools/**`, `justfiles/**`, `moon.yml`              | planner            |
| `AGENTS.md`, `agent/**`, root config, `docs/architecture/**`    | planner            |

Multi-domain dispatch order: platform-ops -> contract -> service -> server/worker -> app-shell -> verification.

Only split work when directory, responsibility, or verification boundaries genuinely differ.

Tooling changes may require platform, contract, service, worker, or app-shell review when the changed command controls those domains.

## 7. Gate Selection

Gate selection is based on changed paths, risk category, and evidence level, not subagent identity.

Use `agent/manifests/gate-matrix.yml` to choose advisory, guardrail, or invariant gates.

Default backend-core development should prefer path-scoped guardrails and `just check-backend-primary`. `just verify` is the default repo-wide backend-core gate when broader confidence is needed, not a requirement that every change run every platform, frontend, desktop, production, or release gate.

P0 invariants and release readiness require executable evidence. Metadata alone is never sufficient.

## 8. Script And Tooling Control Plane

Repository tooling is part of the executable control plane.

`just` is the human command surface. Keep recipes thin.
`moon` is task orchestration. Keep task graph and cache inputs there, not business logic.
`tools/repo-tools` is the Rust repo-control CLI. Put reusable validation, generation, drift, routing, and operational logic there.

Rules:

1. Do not move shell complexity into Rust as opaque `bash -c` strings.
2. External commands must use structured arguments, explicit cwd, clear errors, and preflight checks.
3. Write operations must distinguish dry-run from apply.
4. Secret-handling commands must redact values and avoid writing decrypted material unless explicitly required.
5. Generated directories are read-only; modify sources and regenerate.
6. High-risk operations need plan, preflight, execute, and verify phases.
7. Cross-platform tooling must not assume Linux-only shell behavior unless the command is explicitly scoped to that platform.

## 9. Quality Bar

Use the smallest correct change, but do not confuse small with tactical.

A good change:

1. respects the owning domain boundary
2. keeps source-of-truth singular
3. makes invalid states harder to represent
4. keeps errors visible and actionable
5. adds or updates verification at the same semantic level as the risk
6. avoids new hidden coupling, global state, and duplicated rules

SOLID guidance for this repository:

1. Treat SOLID as boundary discipline, not object-oriented ceremony.
2. Keep each module focused on one reason to change: domain rules in `domain`/`application`, protocol adaptation in `servers`, async recovery in `workers`, and platform concerns in `platform`/`infra`/`ops`.
3. Extend behavior through contracts, ports, traits, adapters, or composition roots; do not modify inner domain code to satisfy an outer transport, database, UI, or deployment concern.
4. Keep trait contracts narrow enough that implementations can substitute safely. Split ports when callers do not need the same capabilities.
5. Depend inward on stable abstractions. Production service code must not depend on concrete infrastructure adapters, app shells, workers, or other services directly.
6. Allow test-only adapters and fixtures when they provide executable evidence, but do not use test exceptions to justify production dependency direction.

Prefer boring, explicit, well-named code. Introduce abstractions only when they remove real duplication or clarify a stable boundary.

Avoid these anti-patterns:

1. metadata-only correctness claims
2. generated-file hand edits
3. shell-in-Rust migrations that keep opaque bash semantics
4. broad rewrites without an invariant-driven reason
5. gate gaming by weakening checks or shrinking coverage
6. swallowing errors to keep workflows green
7. duplicating rules across docs, YAML, scripts, and Rust
8. making optional app-shell dependencies part of backend-core defaults

## 10. Global Hard Constraints

1. Read before changing; evidence before judgment; search before guessing.
2. Fix the smallest causal closed loop that restores the violated invariant.
3. Passing tests are evidence, not the goal; restored behavior and restored invariants are the goal.
4. Verification that was not executed cannot be claimed as passed.
5. Mark uncertainty explicitly; never dress up guesses as conclusions.
6. Do not solve by deleting, skipping, weakening validation, swallowing errors, bypassing gates, or faking success.
7. Do not use metadata, generated files, or prose docs to pretend behavior exists.
8. Do not introduce broad rewrites when a narrow semantic repair is sufficient.
9. Do not add compatibility layers unless there is persisted data, shipped behavior, external consumers, or explicit user requirement.
10. Generated artifact directories are read-only; modify the source and regenerate.
11. Escalate risk when a change affects distributed semantics, authorization, secrets, topology, contract compatibility, generated artifacts, template structure, or release correctness.

## 11. Bug Fix Protocol

For bug reports, failing gates, regressions, and suspicious behavior:

1. Reproduce or localize the failure with code, schema, validator, gate, script, or command-output evidence when feasible.
2. Identify the violated invariant, not only the failing line or symptom.
3. Determine the causal boundary: contract, service, server, worker, platform, infra, app, or cross-boundary.
4. Make the minimal complete repair at that boundary.
5. Add or update regression verification at the same semantic level as the bug.
6. Run relevant gates and state explicitly when a gate was not run.
7. If only a tactical fix is safe, state the residual risk.

## 12. Reporting Evidence

When reporting completion, state:

1. what changed
2. why it restores or improves the relevant invariant
3. what verification ran
4. what was not run
5. any residual risk or uncertainty

Use evidence level terms consistently: `declared`, `checked`, `tested`, `proven`.
