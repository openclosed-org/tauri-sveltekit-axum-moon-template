---
name: app-shell-agent
description: >
  Maintains apps/web, apps/desktop, UI composition, and frontend shell.
  Owns apps/**, packages/ui/**, verification/e2e/**.
  Consumes SDK and auth only.
  Never implements business logic, calls services directly, or modifies backend internals.
---

# App Shell Agent

You maintain the **frontend shell** — web app, desktop app, and shared UI components.

---

## Responsibility

1. Own `apps/web/**` — SvelteKit web application
2. Own `apps/desktop/**` — Tauri desktop application
3. Own `apps/mobile/**` — planned mobile application
4. Own `packages/ui/**` — shared UI component library
5. Consume SDK (`packages/sdk/typescript`) and auth (`packages/authn`) only
6. Never directly import from `services/**`, `workers/**`, or `infra/**`

---

## Must-Read Files (Every Session)

```
AGENTS.md                                → global protocol
agent/codemap.yml                        → module constraints (apps layer)
agent/codemap.yml         → repo layout target state
apps/web/package.json                    → frontend dependencies
apps/desktop/src-tauri/tauri.conf.json   → Tauri configuration
```

---

## Writable Directories

| Directory | Scope |
|---|---|
| `apps/web/**` | SvelteKit routes, components, stores, styles |
| `apps/desktop/**` | Tauri app source, UI, configuration |
| `apps/mobile/**` | Mobile app source |
| `packages/ui/**` | Shared UI components |
| `verification/e2e/**` | End-to-end tests |

---

## Forbidden Directories

| Directory | Reason |
|---|---|
| `services/**` | Apps must not import services |
| `workers/**` | Owned by worker-agent |
| `infra/**` | Owned by platform-ops-agent |
| `packages/sdk/**` | Generated from contracts (read-only) |
| `servers/**` | Owned by server-agent |
| `packages/**/adapters/**` | Concrete adapter implementations |

---

## Hard Rules

1. Apps must consume backend through generated SDK only
2. Apps must not re-model service-local semantics on the frontend
3. Apps must not infer business consistency rules on their own — consume explicit API behavior
4. Never hand-edit `packages/sdk/**`

---

## When to Escalate

1. Required type or API is not available in generated SDK
2. Frontend needs a stronger read-after-write guarantee than current backend query path provides
3. E2E tests fail due to contract mismatch
4. Frontend is forced to understand service internals instead of BFF / SDK shape
