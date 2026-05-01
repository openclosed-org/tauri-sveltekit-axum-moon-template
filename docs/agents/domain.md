# Agent Domain Docs

This file defines how agents should consume repository language and decision docs.

## Read Order

For backend and harness work, read:

1. `AGENTS.md`
2. `docs/architecture/north-star.md`
3. `docs/architecture/harness-philosophy.md`
4. relevant `docs/language/**`
5. accepted ADRs that touch the area
6. executable sources, validators, tests, gates, scripts, and command output

Use `agent/codemap.yml`, `agent/manifests/routing-rules.yml`, and `agent/manifests/gate-matrix.yml` for path ownership and gate selection.

## Language Docs

`docs/language/**` provides vocabulary only. Use its terms in plans, tests, issues, and reviews.

Do not treat language docs as evidence that behavior exists. If a term conflicts with code, validators, tests, gates, or command output, trust executable evidence.

## ADRs

Read ADRs when the task touches architecture, topology, storage, protocol, or cross-boundary behavior.

Only create or update an ADR when the decision is:

1. hard to reverse
2. surprising without context
3. the result of a real trade-off

## Scratch Guidance

`docs/_local/**` is scratch space. Do not use it as a default rule source unless the user or active workflow explicitly points to it.

Promote stable scratch guidance into tracked docs only when it is durable, scoped, and does not claim stronger evidence than exists.

## GitHub Discussions

GitHub Discussions are rationale and RFC material. They are not current repository rules unless promoted into tracked docs or ADRs.
