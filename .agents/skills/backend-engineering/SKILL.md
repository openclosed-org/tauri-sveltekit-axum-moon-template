---
name: backend-engineering
description: >
  Always-on backend engineering quality kernel for Rust/Axum harness work.
  Use when changing or reviewing backend services, servers, workers, contracts,
  repo-tools, platform-facing backend metadata, or backend documentation.
  Guides invariant-first design, boundary discipline, evidence language, and
  template-quality code judgment without overriding ownership skills.
---

# Backend Engineering

Use this skill as the default quality lens for backend harness work.

Workflow skills guide judgment. Ownership skills still decide writable boundaries.

## Objective

Keep the template small, explicit, typed, testable, and production-minded without pretending it is production-proven.

Prefer the smallest design that preserves:

1. backend correctness
2. hard service, protocol, server, worker, and platform boundaries
3. topology-late growth
4. executable evidence
5. template-user simplicity
6. agent resistance to drift

## Decision Kernel

Before changing backend behavior, answer:

1. What invariant must not be violated?
2. Which boundary owns the violated or changed behavior?
3. What is synchronous, asynchronous, durable, idempotent, replayable, or eventually consistent?
4. What is the cheapest correct design that restores the invariant?
5. What evidence can honestly be claimed after this change?

## Boundary Rules

1. `services/**` are business capability libraries by default, not service processes.
2. `servers/**` adapt protocols and compose services; handlers do not own domain logic.
3. `workers/**` advance async state and must make retry, idempotency, checkpoint/replay, dedupe, and recovery explicit when touched.
4. `packages/contracts/**` owns shared protocol shapes before external API or event changes land elsewhere.
5. `platform/model/**` describes platform-level metadata and global defaults; service-local semantics stay in `services/<name>/model.yaml`.
6. Generated artifacts are read-only; change sources and regenerate.
7. Optional app shells and UI packages must not become backend-core prerequisites.

## Quality Rules

1. Represent important invariants in types, constructors, constraints, or final transaction boundaries.
2. Keep expected failures typed and matchable; do not leak internal dependency details to clients.
3. Retries for side-effecting work require durable idempotency or an explicit reason they are safe.
4. Cross-boundary events require a clear atomic state mutation and publication story, usually outbox-driven.
5. Async work must have bounded concurrency, cancellation/shutdown behavior, timeout or backpressure expectations, and observable failure paths when relevant.
6. Do not add traits, managers, adapters, compatibility layers, or generic repositories unless they protect a real boundary or remove proven duplication.
7. Prefer reference-module reuse over new structure, but do not copy business semantics blindly.
8. Fix the smallest causal closed loop; avoid broad rewrites without an invariant-driven reason.

## Evidence Rules

Use evidence terms precisely:

1. `declared`: docs, metadata, manifests, or model files state intent.
2. `checked`: schema, static validation, typecheck, drift check, or boundary check inspected structure.
3. `tested`: automated tests exercised behavior.
4. `proven`: a relevant executed gate or runtime/operational evidence supports the claimed invariant.

Never upgrade a claim from `declared` to `checked`, `tested`, or `proven` without executable evidence.

## When To Deepen Review

Use focused workflow skills only when risk justifies deeper analysis:

1. architecture review, release readiness, or template admission work
2. distributed semantics, replay, idempotency, outbox, or worker recovery changes
3. security, secrets, authorization, or tenant boundary changes
4. broad backend refactors or cross-boundary protocol changes
5. user explicitly asks for top-level backend review or code-quality audit
