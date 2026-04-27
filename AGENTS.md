# AGENTS.md

> Thin coordination protocol for this repository. All agents read this first.
> This file contains stable cross-cutting rules only. Harness philosophy lives in `docs/architecture/harness-philosophy.md`; current-state guidance lives in `docs/README.md`.

## 1. Language Preference

Communication **MUST** be Chinese.

Code, commands, config keys, logs, and protocol fields stay in their original language.

## 2. Default Reading Order

Backend tasks:

1. `AGENTS.md`
2. `docs/architecture/harness-philosophy.md`
3. `agent/codemap.yml`
4. `agent/manifests/routing-rules.yml`
5. `agent/manifests/gate-matrix.yml`

Documentation / audit tasks: also read `docs/README.md`.

## 3. Truth Hierarchy

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

## 4. Routing

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
| `AGENTS.md`, `agent/**`, root config, `docs/architecture/**`    | planner            |

Multi-domain dispatch order: platform-ops -> contract -> service -> server/worker -> app-shell -> verification.

Only split work when directory, responsibility, or verification boundaries genuinely differ.

## 5. Gate Selection

Gate selection is based on changed paths, risk category, and evidence level, not subagent identity.

Use `agent/manifests/gate-matrix.yml` to choose advisory, guardrail, or invariant gates.

Default backend-core development should prefer path-scoped guardrails and `just verify-backend-primary`. `just verify` is the default repo-wide backend-core gate when broader confidence is needed, not a requirement that every change run every platform, frontend, desktop, production, or release gate.

P0 invariants and release readiness require executable evidence. Metadata alone is never sufficient.

## 6. Global Hard Constraints

1. Read before changing; evidence before judgment; search before guessing.
2. Fix the smallest causal closed loop that restores the violated invariant.
3. Passing tests are evidence, not the goal; restored behavior and restored invariants are the goal.
4. Verification that was not executed cannot be claimed as passed.
5. Mark uncertainty explicitly; never dress up guesses as conclusions.
6. Do not solve by deleting, skipping, weakening validation, swallowing errors, bypassing gates, or faking success.
7. Generated artifact directories are read-only; modify the source and regenerate.
8. Escalate risk when a change affects distributed semantics, authorization, secrets, topology, contract compatibility, or release correctness.

## 7. Bug Fix Protocol

For bug reports, failing gates, regressions, and suspicious behavior:

1. Reproduce or localize the failure with code, schema, validator, gate, script, or command-output evidence when feasible.
2. Identify the violated invariant, not only the failing line or symptom.
3. Determine the causal boundary: contract, service, server, worker, platform, infra, app, or cross-boundary.
4. Make the minimal complete repair at that boundary.
5. Add or update regression verification at the same semantic level as the bug.
6. Run relevant gates and state explicitly when a gate was not run.
7. If only a tactical fix is safe, state the residual risk.
