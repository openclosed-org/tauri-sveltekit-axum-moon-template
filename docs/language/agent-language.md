# Agent Language

This file defines agent and documentation workflow terms. It is vocabulary, not runtime evidence.

## Terms

**Ownership skill**
A skill that defines which paths and responsibilities an agent may own.
_Avoid_: workflow skill.

**Workflow skill**
A skill that defines how to perform an engineering activity such as diagnosis, TDD, grilling, or architecture review.
_Avoid_: path owner.

**Agent brief**
A durable task contract that states current behavior, desired behavior, key interfaces, acceptance criteria, owned paths, forbidden paths, and required evidence.
_Avoid_: transient chat summary.

**Current-state doc**
A tracked document that describes how the repository currently works at a stated evidence level.
_Avoid_: roadmap, RFC.

**Target-state doc**
A planning or design document that describes desired future behavior without claiming current evidence.
_Avoid_: current rule.

**Promotion**
The act of moving a stable term, decision, or guidance from scratch space into tracked current docs, language docs, or ADRs.
_Avoid_: copying discussion text.

**Scratch guidance**
Temporary planning or exploratory material under `docs/_local/**`.
_Avoid_: default agent reading path.

**Rejected direction memory**
A durable note explaining a deliberately out-of-scope capability so future triage does not relitigate it.
_Avoid_: current runtime fact.

## Relationships

1. **Workflow skills** guide process, but **Ownership skills** still decide writable boundaries.
2. **Scratch guidance** can become a **Current-state doc** only through explicit **Promotion**.
3. **Agent briefs** should be behavioral and durable; avoid line numbers and stale implementation instructions.
