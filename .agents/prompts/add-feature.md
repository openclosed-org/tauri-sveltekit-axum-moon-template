# Add Feature Prompt

**Purpose:** Guide an agent through adding a new feature module, from contract definition to frontend consumption.
**When to Use:** When you need to add a new capability that spans multiple layers (API endpoint + UI component + backend logic).

---

## Prerequisites

- [ ] Read `AGENTS.md` for execution protocol and hard constraints
- [ ] Read `.agents/rubrics/boundary-compliance.md` for layer import rules
- [ ] Read `.agents/playbooks/create-feature.md` for the detailed 5-step flow
- [ ] Run `just verify` to confirm current state is clean

---

## Steps

### 1. Identify Feature Scope

Determine which layers the feature touches:
- **contracts** — Does it expose data across boundaries? Define DTOs in `packages/contracts/api/`
- **domain** — Does it require a new capability? Define port trait in `packages/core/domain/`
- **usecases** — Does it have business logic? Implement service in `packages/core/usecases/`
- **adapters** — Does it need storage or host integration? Implement in `packages/adapters/`
- **servers** — Does it need HTTP routes? Add to `servers/api/`
- **hosts** — Does it need Tauri commands? Add to `packages/adapters/hosts/tauri/`
- **frontend** — Does it need UI? Create SvelteKit route/component

### 2. Follow create-feature Playbook

Execute the 5-step flow from `.agents/playbooks/create-feature.md`:

1. **Define Contracts** — Add DTO structs with `#[derive(Serialize, Deserialize, TS, utoipa::ToSchema)]`
2. **Implement Domain Port** — Define framework-agnostic trait in `packages/core/domain/`
3. **Implement Usecases Service** — Business logic with local input/output types (NOT contracts_api types)
4. **Implement Adapter** — Storage, host, or external adapter as appropriate
5. **Implement Host + Frontend** — Route handlers, Tauri commands, SvelteKit routes

### 3. Boundary Compliance Checklist

Per `.agents/rubrics/boundary-compliance.md`:

- [ ] `domain` does NOT import adapters, hosts, contracts, servers
- [ ] `usecases` does NOT import contracts_api types (defines its own)
- [ ] `contracts` does NOT import domain, usecases, adapters, servers
- [ ] Adapters reference domain port traits, not concrete types
- [ ] Tauri commands delegate to usecases, no inline business logic
- [ ] Server routes only do DTO mapping + delegation

---

## Verification

```bash
# Generate TypeScript types
just typegen

# Full quality check
just verify

# Rust tests
cargo test
```

Manual checks:
- [ ] No `unwrap()` in production code
- [ ] No `console.log` in frontend code
- [ ] No new compiler warnings
- [ ] `frontend/generated/` files committed
- [ ] Functions ≤ 50 lines, files ≤ 800 lines

---

## References

- `.agents/playbooks/create-feature.md` — Detailed 5-step flow
- `.agents/rubrics/boundary-compliance.md` — Layer import rules
- `.agents/rubrics/code-review.md` — Code quality checklist
- `.agents/rubrics/task-completion.md` — Completion criteria
