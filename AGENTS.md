# AGENTS.md

> Thin coordination protocol for this repository. All agents read this first.
> This file contains cross-cutting rules, constraints, and indexes — not descriptions of current state.
> Current-state guidance lives in `docs/README.md`; machine-readable rules live in `agent/codemap.yml`.

## 1. Language Preference

Communication defaults to English. To switch, add to your agent configuration:

```yaml
communication_language: zh-CN
```

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

| Path | Subagent |
|------|----------|
| `platform/model/**`, `platform/schema/**`, `infra/**`, `ops/**` | platform-ops-agent |
| `packages/contracts/**`, `docs/contracts/**` | contract-agent |
| `services/**` | service-agent |
| `servers/**` | server-agent |
| `workers/**` | worker-agent |
| `apps/**`, `packages/ui/**` | app-shell-agent |
| `AGENTS.md`, `agent/**`, root config | planner |

Multi-domain dispatch order: platform-ops → contract → service → server/worker → app-shell → final verification.
Only split when directory, responsibility, or verification boundaries genuinely differ.

## 6. Global Hard Constraints

1. Read before changing; evidence before judgment; search before guessing.
2. Prioritize minimal closed loops; no unrelated refactoring.
3. Verification that wasn't executed cannot be claimed as passed.
4. Mark uncertainty explicitly; never dress up guesses as conclusions.
5. "Solving" by deleting, skipping, swallowing errors, or faking success is forbidden.
6. Generated artifact directories are read-only; modify the source and regenerate.
7. Escalate risk when: the change conflicts with architecture ADRs, spans multiple core directories, changes critical dependencies, or involves 4+ subagents with complex dependencies.
