# Agent Docs

`docs/agents/**` is the durable guidance layer for agent context flow. It explains which document type to read, when to read it, and how it relates to skills and manifests.

These docs do not override executable sources, accepted ADRs, `AGENTS.md`, `agent/codemap.yml`, or `agent/manifests/**`.

## Context Flow

| Layer | Files | Purpose | Read When |
|---|---|---|---|
| Global protocol | `AGENTS.md` | Stable cross-cutting rules: evidence, routing posture, quality bar, reporting | Every agent-assisted task |
| Navigation maps | `agent/codemap.yml`, `agent/manifests/routing-rules.yml`, `agent/manifests/gate-matrix.yml` | Path ownership, dispatch, gate selection | Before deciding owner, writable boundary, or verification |
| Architecture doctrine | `docs/architecture/north-star.md`, `docs/architecture/harness-philosophy.md`, accepted ADRs | Durable architectural constraints and trade-offs | Backend, harness, cross-boundary, or architecture-sensitive work |
| Vocabulary | `docs/language/**` | Shared terms only; not runtime evidence | Plans, reviews, task briefs, naming, ambiguity resolution |
| Agent docs | `docs/agents/**` | Context-consumption, skill-authoring, and task-brief conventions | Editing skills, writing durable briefs, or resolving context-flow questions |
| Skills | `.agents/skills/**` | Triggerable ownership or workflow instructions | When the task matches a skill description |
| Scratch | `docs/_local/**` | Temporary plans and exploratory notes | Only when explicitly referenced by the user or active workflow |

## Skill Types

| Type | Examples | Purpose | Must Not Do |
|---|---|---|---|
| Ownership skill | `service-agent`, `server-agent`, `worker-agent`, `contract-agent`, `platform-ops-agent`, `app-shell-agent`, `planner` | Define writable paths, forbidden paths, and domain responsibility | Override executable evidence or write outside its boundary without escalation |
| Workflow skill | `harness-diagnose`, `harness-tdd`, `harness-grill`, `harness-architecture-review`, `harness-zoom-out`, `backend-engineering`, `skill-authoring` | Define how to perform a task or review | Override ownership boundaries or become a second source of truth for file maps |

## Files Here

1. `domain.md` defines how agents consume docs, ADRs, executable evidence, and scratch guidance.
2. `skill-authoring.md` defines how repository skills should be written and reviewed.
3. `agent-brief-format.md` defines durable task brief structure for issue, PRD, or AFK-agent workflows.
4. `language-candidates-template.md` defines the scratch vocabulary capture template.

## Keep Clean

1. Put stable repository rules in `AGENTS.md`, `agent/codemap.yml`, or accepted ADRs, not repeated across every skill.
2. Put volatile plans in `docs/_local/**`, not tracked agent docs.
3. Put path-to-owner and gate logic in `agent/manifests/**`, not prose copies.
4. Keep `SKILL.md` files triggerable and short; move deeper reference material to adjacent files only when needed.
