# Agent Domain Docs

This file defines how agents consume repository docs without turning prose into false evidence.

## Reading Model

Start with the narrowest durable source that can answer the task:

| Need | Read |
|---|---|
| Global rules and reporting expectations | `AGENTS.md` |
| Path ownership or dependency direction | `agent/codemap.yml` and `agent/manifests/routing-rules.yml` |
| Gate selection | `agent/manifests/gate-matrix.yml` |
| Architecture constraints | `docs/architecture/north-star.md`, `docs/architecture/harness-philosophy.md`, relevant ADRs |
| Shared terminology | Relevant `docs/language/**` |
| Skill behavior | Relevant `.agents/skills/**/SKILL.md` |
| Current behavior | Code, schemas, validators, tests, gates, scripts, and command output |

Do not read every document by default. Add context when the task crosses a boundary, touches a durable decision, or a term is ambiguous.

## Evidence Boundaries

1. Language docs provide vocabulary only.
2. Agent docs provide operating conventions only.
3. Skills provide triggers, boundaries, and workflow guidance only.
4. Manifests route work and recommend gates, but do not prove runtime semantics.
5. Executable sources and command output decide `checked`, `tested`, and `proven` claims.

## ADRs

Read ADRs when a task touches architecture, topology, storage, protocol, or cross-boundary behavior.

Create or update an ADR only when the decision is hard to reverse, surprising without context, and the result of a real trade-off.

## Scratch Guidance

`docs/_local/**` is scratch space. Do not use it as a default rule source unless the user or active workflow explicitly points to it.

Promote stable scratch guidance into tracked docs only when it is durable, scoped, and does not claim stronger evidence than exists.

## GitHub Discussions

GitHub Discussions are rationale and RFC material. They are not current repository rules unless promoted into tracked docs or ADRs.
