# Harness Language

This file defines repository-wide terms. It is vocabulary, not runtime evidence.

## Terms

**Harness**
A coordination layer that routes work, points to declared metadata and executable evidence, and recommends gates.
_Avoid_: framework, platform proof, production system.

**Backend-core**
The default backend path that must remain runnable and verifiable without optional app shells, desktop code, mobile shells, UI packages, or advanced cluster features.
_Avoid_: full stack, complete platform.

**Reference chain**
A narrow current path used to learn, copy, and verify cross-layer behavior at its stated evidence level.
_Avoid_: production-ready chain, complete production chain.

**Reference anchor**
The service or chain used as the default learning and copy target until another path reaches the same evidence level.
_Avoid_: golden source, proof source.

**Declared metadata index**
YAML or model metadata that records intended shape, ownership, or defaults but does not prove behavior.
_Avoid_: source of truth, truth source, behavior proof.

**Declared semantics index**
Service-local metadata that summarizes intended commands, events, queries, ownership, consistency, and idempotency semantics.
_Avoid_: service truth source, semantic proof.

**Executable evidence**
Code, schemas, validators, tests, gates, scripts, logs, or command output that can support a checked, tested, or proven claim.
_Avoid_: documentation claim, metadata claim.

**Topology-late**
The rule that service semantics are designed before choosing whether they run embedded, as workers, or as independently deployed processes.
_Avoid_: microservice-first, monolith-only.

**Service library**
A `services/*` package that owns domain logic and application semantics without owning process entrypoints.
_Avoid_: service process, HTTP service by default.

**Composition root**
A server or worker entrypoint that wires service libraries to runtime adapters, transport, configuration, or scheduling.
_Avoid_: domain owner.

**Optional lane**
A capability that can exist as a documented direction or profile without becoming a backend-core prerequisite.
_Avoid_: default dependency, required platform path.

**Capability slot**
A named place for a future or optional capability whose semantics are intentionally reserved but not claimed as current behavior.
_Avoid_: implemented feature, generated placeholder.

**Admission closure**
The work needed before a stronger release or readiness claim: close the smallest causal gaps and provide matching evidence.
_Avoid_: feature expansion, platform completion.

**Drift signal**
A phrase, file, or behavior that suggests target state, metadata, or discussion is being mistaken for current executable behavior.
_Avoid_: harmless wording.

## Relationships

1. The **Harness** protects **backend-core** by keeping optional lanes out of default gates.
2. A **Reference chain** can use a **Reference anchor**, but only executable evidence can raise its claim above `declared`.
3. **Topology-late** keeps **Service libraries** separate from **Composition roots**.
4. A **Capability slot** can describe intended shape without becoming executable evidence.
