# Maintainer Decision Guide

Use this guide to triage issues and PRs without relying on ad hoc architectural judgment.

The goal is not to make every change heavy. The goal is to make user type, contribution path, risk, evidence, and merge criteria explicit.

## Step 1: Identify the User Type

Is this from:

1. a template user; or
2. an upstream contributor / agent?

Template-user questions should not be forced through upstream contribution rules.

Upstream changes must declare path, risk, and evidence.

If unclear, ask for classification before reviewing implementation details.

## Step 2: Classify the Contribution Path

Use one path:

- Core Path
- Reference Chain
- Topology Profile

If unclear, label `needs-classification` and request a narrower scope.

Core Path is for small, low-risk improvements such as docs hygiene, tests, local workflow, Rust cleanup, and small internal refactors.

Reference Chain is for backend correctness changes around `counter-service`, contracts, CAS, idempotency, transactional outbox, relay, projector, replay, rebuild, and read models.

Topology Profile is for deployment and cross-cutting infrastructure such as SOPS, Kustomize, Flux, K3s, NATS, Valkey, observability, platform model topology, and runbooks.

## Step 3: Classify Risk

Low risk:

- docs hygiene
- tests that do not change behavior
- local workflow that does not change default runtime dependencies
- small internal refactor

Medium risk:

- service internals
- contracts implementation without public shape changes
- generated checks
- platform validation
- worker internals without new distributed semantics

High risk:

- public API contracts
- event contracts
- idempotency
- CAS
- outbox
- tenant/auth
- topology defaults
- gate matrix
- directory semantics
- generated artifact rules
- default runtime dependencies

High-risk changes require a prior issue, contribution proposal, or architecture RFC before merge.

## Step 4: Check Evidence

Use these terms:

- `declared`: metadata or prose only
- `checked`: schema validation, static validation, typecheck, drift check, or boundary check
- `tested`: unit, integration, contract, replay, resilience, or end-to-end tests
- `proven`: executed gate or test evidence appropriate for the claimed invariant

Do not merge high-risk changes with only declared evidence.

Do not accept claims that a gate passed unless the PR lists the exact command and result from this change context.

## Step 5: Check Forbidden Moves

Reject or request changes if the PR:

- weakens gates to pass CI
- hand-edits generated artifacts
- deletes valid topology/profile/capability-slot work without classification
- changes defaults without discussion
- claims unrun gates passed
- turns future intent into current fact
- introduces product-specific logic as generic template design
- makes non-default frontend, desktop, production, release, or platform-full gates mandatory for ordinary backend-core changes

## Step 6: Check Runtime Minimalism And Semantic Completeness

Do not interpret single-VPS friendliness as no workers, no outbox, no contracts, no telemetry, no secrets discipline, no deployment model, or no future topology path.

Do not interpret microservice readiness as all distributed tools running by default.

The expected balance is:

- default runtime path stays lightweight
- semantic boundaries remain explicit
- local substitutes exist for inner-loop development
- real adapters and runtime profiles are enabled when justified
- enabling a real tool should not rewrite business code

## Step 7: Merge Rule

Merge only when:

- user type is clear
- contribution path is clear
- risk is acceptable
- evidence matches risk
- docs do not overclaim
- gates are not weakened
- generated artifacts were not hand-edited
- the PR reduces uncertainty

If these are not true, request changes or redirect to an issue/RFC.
