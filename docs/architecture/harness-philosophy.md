# Harness Philosophy

This repository uses the agent harness as a coordination layer. It is not a formal architecture constitution, a proof system, or a replacement for executable verification.

## Boundary

The harness may:

1. map changed paths to likely owners
2. point agents at source-of-truth files
3. recommend gates based on path, risk, and evidence needs
4. record durable coordination rules that should remain stable across tasks

The harness must not:

1. claim semantic correctness from YAML metadata alone
2. turn every local change into a full release gate
3. make subagent identity the reason a gate is required
4. duplicate volatile implementation details that are better read from code, tests, scripts, or schemas

## Truth Hierarchy

When sources disagree, use this order:

1. code, schemas, validators, executable tests, gates, and command output
2. generated artifacts only when they are produced by the current source and verified for drift
3. platform and service metadata such as `platform/model/**` and `services/*/model.yaml`
4. coordination manifests such as `agent/codemap.yml`, `routing-rules.yml`, and `gate-matrix.yml`
5. prose documentation and historical notes

Metadata can describe intent, ownership, topology, or semantic summaries. It cannot prove the implementation satisfies those semantics.

## Metadata Is Not Proof

`agent/codemap.yml` is a navigation map. It should stay small and point to boundaries, sources of truth, generated-readonly locations, anti-patterns, and modification order.

`services/<name>/model.yaml` is a service semantic summary. It should help reviewers find declared ownership, commands, events, queries, consistency expectations, idempotency notes, and replay assumptions. It is not a formal proof.

`platform/model/**` describes platform shape and global deployable metadata. It does not prove runtime safety, state correctness, or release readiness by itself.

Do not upgrade a claim from `declared` to `checked`, `tested`, or `proven` by editing metadata alone.

## Evidence Levels

Use these terms consistently:

1. `declared`: written in metadata or docs; useful for navigation, not evidence of behavior
2. `checked`: validated by a schema, static validator, drift check, typecheck, or boundary check
3. `tested`: exercised by unit, integration, contract, replay, or end-to-end tests
4. `proven`: supported by an explicit gate or test run that is appropriate for the claimed invariant and cited in the change record

An agent may only raise evidence level by referencing executable evidence: test names, gate names, scripts, or command output that actually ran.

## Gate Strength

Gate strength is separate from subagent identity.

1. `advisory`: gives signal but does not block. Use for early warnings, optional lanes, exploratory checks, or non-default shells.
2. `guardrail`: may block a PR. Use for boundary checks, contract drift, default backend-core static checks, and path-scoped validation that protects normal development.
3. `invariant`: reserved for P0 correctness and release readiness. Use only when the change affects data correctness, distributed semantics, ownership, authorization, secrets, topology, replay, or production delivery paths.

P0 invariants must be demonstrated by executable tests, gates, or command-output evidence. YAML declarations can point to the invariant, but they do not prove it.

## Default Backend Core

The default backend-core lane should remain light enough for ordinary development. Prefer `just check-backend-primary` and path-scoped guardrails before requiring repo-wide or release gates.

`just verify` is the default repo-wide backend-core gate when broader confidence is needed, but it should not imply that every change must run platform, frontend, desktop, production, or release gates.

Release and full gates may be heavier. They belong to release preparation, P0 invariant changes, platform topology changes, distributed semantics changes, or explicit reviewer request.

## Applying Gates

Choose gates from changed paths, risk category, and evidence level:

1. path decides what changed
2. risk decides how strong the gate must be
3. evidence level decides what can be claimed afterward
4. subagent identity only helps route work; it does not make a gate required

If a gate was not run, say so. If a gate is advisory, do not describe it as blocking. If a gate is a release or invariant gate, explain the P0 or release risk that justifies it.
