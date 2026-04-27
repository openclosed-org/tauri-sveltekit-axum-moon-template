# AGENTS.md

> Thin coordination protocol for this repository. All agents read this first.
> This file contains cross-cutting rules, constraints, and indexes — not descriptions of current state.
> Current-state guidance lives in `docs/README.md`; machine-readable rules live in `agent/codemap.yml`.

## 1. Language Preference

Communication **MUST** be Chinese

Code, commands, config keys, logs, and protocol fields always remain in their original language.

## 2. Default Reading Order

Backend tasks:

1. `AGENTS.md`
2. `agent/codemap.yml`
3. `agent/manifests/routing-rules.yml`
4. `agent/manifests/gate-matrix.yml`
5. `docs/operations/counter-service-reference-chain.md`

Documentation / audit tasks: also read `docs/README.md`.

## 3. Source-of-Truth Priority

When determining the current state, gather evidence in this order:

1. Code, schemas, validators, gates, scripts
2. `agent/codemap.yml`
3. `agent/manifests/routing-rules.yml`
4. `agent/manifests/gate-matrix.yml`
5. `docs/operations/counter-service-reference-chain.md`
6. `docs/adr/**` and `.agents/skills/*/SKILL.md`

Hard rules:

1. When docs conflict with code, trust code and executable verification.
2. Never infer a file or module exists solely from target-state documentation.
3. Conclusions about the current state must point to a real file, directory, or command output.

## 4. Planner Responsibilities

**Does**: understand goals, audit directories, dispatch subagents per `routing-rules.yml`, advance changes in dependency order, converge results.
**Does NOT**: design non-existent modules, merge multi-domain patches into one, replace gates/scripts with prose.

## 5. Routing

Full mapping lives in `agent/manifests/routing-rules.yml`. Quick reference:

| Path                                                            | Subagent           |
| --------------------------------------------------------------- | ------------------ |
| `platform/model/**`, `platform/schema/**`, `infra/**`, `ops/**` | platform-ops-agent |
| `packages/contracts/**`, `docs/contracts/**`                    | contract-agent     |
| `services/**`                                                   | service-agent      |
| `servers/**`                                                    | server-agent       |
| `workers/**`                                                    | worker-agent       |
| `apps/**`, `packages/ui/**`                                     | app-shell-agent    |
| `AGENTS.md`, `agent/**`, root config                            | planner            |

Multi-domain dispatch order: platform-ops → contract → service → server/worker → app-shell → final verification.
Only split when directory, responsibility, or verification boundaries genuinely differ.

## 6. Global Hard Constraints

1. Read before changing; evidence before judgment; search before guessing.
2. Prioritize the smallest causal closed loop: identify the violated invariant and root-cause boundary first, then make the minimal complete repair; no unrelated refactoring.
3. Passing tests are evidence, not the goal. The goal is restored behavior, restored invariants, and executable verification that would fail without the fix.
4. Verification that wasn't executed cannot be claimed as passed.
5. Mark uncertainty explicitly; never dress up guesses as conclusions.
6. "Solving" by deleting, skipping, weakening validation, swallowing errors, bypassing gates, or faking success is forbidden.
7. Generated artifact directories are read-only; modify the source and regenerate.
8. Escalate risk when: the change conflicts with architecture ADRs, spans multiple core directories, changes critical dependencies, affects distributed semantics, or involves 4+ subagents with complex dependencies.

## 7. Bug Fix and Repair Protocol

Use this protocol for bug reports, failing gates, regressions, and suspicious behavior:

1. Reproduce or localize the failure with code, schema, validator, gate, script, or command-output evidence when feasible.
2. Identify the violated invariant, not only the failing line, failing test, or surface symptom.
3. Determine the causal boundary: contract, service, server, worker, platform, infra, app, or cross-boundary.
4. Fix the smallest causal closed loop that restores the invariant.
5. Add or update regression verification at the same semantic level as the bug.
6. Run the relevant gates; state explicitly when a gate was not run.
7. If only a tactical fix is safe in the current turn, state the residual architectural or operational risk explicitly.

Bug-fix shortcuts are forbidden:

1. Do not delete, skip, or weaken tests to pass.
2. Do not swallow errors or replace failures with defaults unless the domain explicitly defines that fallback.
3. Do not edit generated artifacts directly.
4. Do not move logic across ownership boundaries to avoid fixing the owner.
5. Do not satisfy a gate by bypassing the behavior the gate was meant to protect.
6. Do not optimize for the smallest diff; optimize for the smallest correct causal repair.

Escalate instead of patching locally when a bug affects distributed semantics, including consistency, idempotency, retry, checkpoint/replay, event ordering, outbox delivery, projection rebuildability, ownership, authorization, secrets, topology, or contract compatibility.
