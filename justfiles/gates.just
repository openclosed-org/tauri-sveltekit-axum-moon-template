# Gates — unified quality entrypoints for humans and agents

# Local gate: strict mode blocks commits that violate boundaries (Phase 2)
gate-local:
    bun run scripts/gate.ts local --mode strict

# Pre-push gate: strict mode blocks pushes that fail quality gates (Phase 2)
gate-prepush:
    bun run scripts/gate.ts prepush --mode strict

# Phase 1 existence/import validation gate
gate-existence MODE='warn':
    bun run scripts/validate-existence.ts --mode {{MODE}}

# Phase 1 import dependency validation gate
gate-imports MODE='warn':
    bun run scripts/validate-imports.ts --mode {{MODE}}

# CI gate: strict once checks leave local warn-only mode
gate-ci:
    bun run scripts/gate.ts ci --mode strict

# Release gate: final strict entry before packaging/release
gate-release:
    bun run scripts/gate.ts release --mode strict

# Scoped gate: run gates specific to a subagent's domain
gate-scoped AGENT:
    bun run scripts/run-scoped-gates.ts {{AGENT}}
