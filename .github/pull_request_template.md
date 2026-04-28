## Contribution Path

Choose one:

- [ ] Core Path
- [ ] Reference Chain
- [ ] Topology Profile

## Summary

What changed?

- 

## Problem / Invariant

What problem does this solve, or what invariant does it protect?

- 

## Risk Level

- [ ] Low
- [ ] Medium
- [ ] High

High-risk areas include:

- public API contracts
- event contracts
- idempotency / CAS / outbox semantics
- tenant / auth boundaries
- topology defaults
- gate matrix
- generated artifact rules
- directory semantics
- default runtime dependencies

## Evidence Level

- [ ] declared
- [ ] checked
- [ ] tested
- [ ] proven

Use these terms consistently:

- `declared`: metadata or prose only
- `checked`: schema validation, static validation, typecheck, drift check, or boundary check
- `tested`: unit, integration, contract, replay, resilience, or end-to-end tests
- `proven`: executed gate or test evidence appropriate for the claimed invariant

## Verification

Commands run:

```bash
# paste exact commands
```

Result summary:

```text
# paste result summary
```

Commands not run and why:

```text
# paste here
```

Only claim checks passed if they actually ran in this change context.

## Contract / Event / Topology / Gate Impact

- [ ] Changes public API contracts
- [ ] Changes event contracts
- [ ] Changes generated artifacts
- [ ] Changes topology profiles
- [ ] Changes gate behavior
- [ ] Changes service ownership
- [ ] Changes default runtime dependencies
- [ ] None of the above

## Agent Use

- [ ] No agent was used
- [ ] Agent was used; output was reviewed by a human maintainer/contributor

If an agent was used, summarize what it changed and what you verified manually.

## Reviewer Focus

What should reviewers inspect most carefully?

- 
