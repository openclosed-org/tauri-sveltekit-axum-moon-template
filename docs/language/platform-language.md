# Platform Language

This file defines platform and delivery vocabulary. It is vocabulary, not runtime evidence.

## Terms

**Platform model**
The platform-level declared metadata index for deployables, topology, secrets shape, workflows, and global defaults.
_Avoid_: platform truth source, behavior proof.

**Service-local model**
The service-owned declared semantics index in `services/<name>/model.yaml`.
_Avoid_: platform-owned service semantics.

**Secret shape**
The declared names, required environment, and injection expectations for runtime secrets.
_Avoid_: decrypted secret source.

**Canonical cluster shape**
The declared deployment shape intended for cluster profiles, typically compatible with SOPS, Kustomize, and Flux.
_Avoid_: required local development path.

**GitOps direction**
The delivery direction for cluster profiles, not a backend-core or host-process development prerequisite.
_Avoid_: mandatory local dependency.

**Overlay**
A deployment customization layer for a profile or environment.
_Avoid_: semantic override.

**Generated artifact**
An output derived from source metadata or schema that must not be hand-edited.
_Avoid_: source file.

**Drift check**
A validation that generated or declared artifacts still match their source inputs.
_Avoid_: semantic proof unless the behavior is also tested or proven.

## Relationships

1. The **Platform model** can declare deployable and topology shape, but **Service-local models** own service semantics.
2. **Canonical cluster shape** and **GitOps direction** must not become backend-core local prerequisites.
3. A **Generated artifact** supports stronger evidence only when a **Drift check** confirms it was produced from current sources.
