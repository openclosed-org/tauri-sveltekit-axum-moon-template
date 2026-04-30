# Backend-Core Requires App Shell

## Decision

Do not make `apps/**`, `packages/ui/**`, desktop, mobile, SvelteKit, Tauri, or frontend e2e checks prerequisites for backend-core commands by default.

## Why This Is Out Of Scope

The repository is backend-first. App shells are optional surfaces that consume SDK/auth shapes; they must not become required for root backend development, backend verification, or template adoption.

If app-shell dependencies leak into backend-core commands, backend-only adopters inherit unnecessary frontend and desktop complexity.

## Reconsideration Criteria

Only add an app-shell requirement to a backend task when the task explicitly changes app-shell behavior or an end-to-end product flow whose evidence requires the shell.

Default backend-core gates must remain independent.

## Related Guidance

1. `AGENTS.md`
2. `docs/README.md`
3. `.agents/skills/app-shell-agent/SKILL.md`
