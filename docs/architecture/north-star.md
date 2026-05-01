# Architecture North Star

This document defines the scale calibration for this repository. It is the short, durable rule set used before reading longer operation docs, ADRs, or GitHub Discussions.

## Purpose

This repository is a backend-first Rust/Axum harness template. It should be small enough to run on a single VPS, but structured so that distributed, service-oriented growth does not require rewriting core semantics.

The goal is not to look like a large platform on day one. The goal is to make the smallest correct backend path explicit, typed, testable, and able to grow along known seams.

## Objective Function

Choose the smallest design that preserves all of these properties:

1. backend correctness
2. hard service and protocol boundaries
3. topology-late deployment growth
4. operational clarity
5. template-user simplicity
6. executable evidence
7. agent resistance to drift

If two designs are both correct, prefer the one with fewer moving parts, fewer names, fewer generated surfaces, and clearer verification.

## Core Tensions

The repository intentionally holds these tensions:

1. single VPS friendly, but not toy architecture
2. production-minded, but not production-proven by claim
3. service-oriented semantics, but not microservice-per-process by default
4. metadata-aware, but never metadata-proven
5. agent-governed, but not checklist-driven ceremony
6. platform-expandable, but backend-core independent of optional shells and advanced infrastructure

When a document or change removes one side of a tension, it is probably drifting.

## Non-Negotiable Rules

1. `services/*` are business libraries by default.
2. `servers/*` and `workers/*` are composition and runtime entrypoints.
3. Contracts change before external protocol shapes change.
4. State mutation, idempotency intent, outbox write, and replay assumptions must be explicit.
5. `counter-service` is the default backend reference anchor until another chain reaches the same evidence level.
6. Optional app shells, desktop code, UI packages, Dapr, Cilium, Gateway API, and full GitOps delivery must not become backend-core prerequisites.
7. Generated artifacts are read-only; modify sources and regenerate.
8. Documentation, YAML, manifests, and generated artifacts do not prove behavior.

## Evidence Hierarchy

Use this order when deciding current state:

1. code, schemas, validators, tests, gates, scripts, and command output
2. generated artifacts only when produced from current sources and checked for drift
3. `platform/model/**` and `services/*/model.yaml` as declared metadata indexes
4. `agent/**` and `.agents/**` routing or skill guidance
5. prose documentation
6. GitHub Discussions, RFCs, and scratch notes

Use evidence words precisely:

1. `declared`: written in metadata, docs, or configuration
2. `checked`: validated by schema, static check, linter, or drift gate
3. `tested`: exercised by automated tests or integration checks
4. `proven`: demonstrated by the relevant runtime, operational, or release evidence

Do not raise a claim above its evidence level.

## Scale Calibration

Default backend path:

```text
counter-service library
  -> web-bff composition root
  -> CAS + idempotency intent + event_outbox
  -> outbox-relay worker
  -> projector worker
  -> replayable read model
```

Default engineering path:

```text
declared platform metadata
  -> secrets shape
  -> overlay / deployable shape
  -> GitOps direction for cluster profiles
  -> runbook / gate evidence
```

This is a reference chain, not a claim that every production concern is already closed.

## Technology Selection

Prefer technology that improves the default path without forcing all adopters into a larger platform. A technology belongs in the default path only when it has:

1. a clear owner
2. a narrow reason to exist
3. a migration story from the current path
4. path-scoped verification
5. no hidden frontend, desktop, cluster, or vendor prerequisite for backend-core work

Otherwise keep it as an optional lane, ADR direction, or deferred capability.

## Documentation Rules

Current-state docs must protect readers from confusing target state with behavior.

1. Say `declared metadata index` for YAML and platform models.
2. Say `reference chain` unless runtime evidence supports a stronger operational claim.
3. Mark deferred, reserved, or target-state paths explicitly.
4. Do not keep placeholder docs that describe future generated files as if they exist.
5. Split any document that tries to explain principles, current state, target state, implementation paths, file maps, and roadmap in one place.

Default docs should protect the correct path, not preserve every useful thought.

## GitHub Discussions

GitHub Discussions are design conversation and historical rationale. They are not current repository rules unless promoted into tracked docs or ADRs.

When a Discussion conflicts with executable sources, accepted ADRs, `AGENTS.md`, or this document, do not follow the Discussion.

## Red Flags

Treat these as drift signals:

1. YAML described as proof of behavior
2. generated or target-state paths edited as if they were current runtime facts
3. optional platform features becoming backend-core requirements
4. services gaining process entrypoints to look more microservice-like
5. read models described as authoritative state
6. local `.env` becoming the backend reference path
7. worker multi-replica behavior claimed without durable ownership, checkpoint, dedupe, and recovery evidence
8. long docs that mix architecture doctrine with stale file maps
