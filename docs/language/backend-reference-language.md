# Backend Reference Language

This file defines terms for the default backend reference path. It is vocabulary, not runtime evidence.

## Terms

**Counter reference chain**
The default backend reference chain centered on `counter-service`, `web-bff`, `event_outbox`, `outbox-relay`, and `projector`.
_Avoid_: production reference chain, complete production sample.

**Authoritative business state**
The write-side state owned by the service boundary for business decisions.
_Avoid_: read model truth source.

**Replayable read model**
A derived view that can be rebuilt from accepted events or outbox records.
_Avoid_: authoritative state.

**Idempotency intent**
The declared or implemented expectation that repeated equivalent commands can be handled safely.
_Avoid_: production-grade idempotency unless transaction and recovery evidence support it.

**CAS mutation**
A compare-and-swap state transition that protects a versioned write path from lost updates.
_Avoid_: distributed lock.

**Outbox record**
A durable event delivery record written with state mutation intent so async workers can relay or replay effects.
_Avoid_: message bus event unless it has actually been published.

**Relay worker**
A worker that moves outbox records toward a bus or downstream consumer with explicit retry and delivery semantics.
_Avoid_: proof of exactly-once delivery.

**Projector worker**
A worker that builds or rebuilds read models from event or outbox sources.
_Avoid_: write-side service.

**Single-replica reference path**
A worker/runtime path whose current evidence is scoped to one active replica.
_Avoid_: multi-replica proven behavior.

## Relationships

1. The **Counter reference chain** is the default learning path, not a production-readiness claim.
2. **Authoritative business state** can feed an **Outbox record**; a **Projector worker** builds a **Replayable read model** from that source.
3. **Idempotency intent** must not be described as a stronger invariant than current transaction, recovery, and test evidence support.
