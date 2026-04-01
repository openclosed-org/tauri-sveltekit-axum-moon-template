# Pitfalls Research

**Domain:** Agent-Native Cross-Platform Application Engineering Base
**Researched:** 2026-04-01
**Confidence:** HIGH

## Critical Pitfalls

### Pitfall 1: Directory Boundary Violation (core depends on host)

**What goes wrong:** Core domain crate imports Tauri/browser APIs.
**Why:** Convenience — faster than creating proper adapter.
**How to avoid:** CI gate for forbidden imports; agent rubric: boundary-compliance.
**Phase:** Phase 1 (directory structure alignment)

### Pitfall 2: Hand-written Type Drift

**What goes wrong:** Rust types and TS interfaces silently diverge.
**Why:** Developer updates one side, forgets the other.
**How to avoid:** Contracts single truth source, CI `repo:contracts-check`, 禁止手写 mirror DTO.
**Phase:** Phase 2 (contracts/typegen)

### Pitfall 3: Host Adapter Carrying Business Logic

**What goes wrong:** Tauri command handler contains business rules.
**Why:** Tempting to put logic "right there."
**How to avoid:** Tauri command 只做 bridge; Feature module pattern.
**Phase:** Phase 3 (runtime boundary)

### Pitfall 4: Feature Coupling Without Contracts

**What goes wrong:** Feature A imports Feature B internals.
**Why:** Faster than defining proper contract interface.
**How to avoid:** Features communicate through usecases/contracts only.
**Phase:** Phase 2-3

### Pitfall 5: Secret Leakage Through Config

**What goes wrong:** Secrets end up in logs, serialized to frontend, or hardcoded.
**Why:** Config struct gets serialized, debug logging prints full config.
**How to avoid:** secret 与 config 分离; redacted Debug impl; CI secret scan.
**Phase:** Phase 1+

### Pitfall 6: Svelte 4 Syntax in Svelte 5 Project

**What goes wrong:** New code uses `$:` instead of Runes.
**Why:** Developer familiarity with Svelte 4.
**How to avoid:** 禁止 `$:`; lint rule; svelte-autofixer.
**Phase:** Phase 4

### Pitfall 7: Skipping Contracts Before Feature Work

**What goes wrong:** Feature starts before contracts defined, result: ad-hoc types.
**Why:** Eagerness to build features.
**How to avoid:** Playbook: 先更新 contracts 再改调用方.
**Phase:** Phase 2

## Technical Debt Patterns

| Shortcut | Long-term Cost | When Acceptable |
|----------|----------------|-----------------|
| Skipping typegen setup | Type drift, runtime errors | Never for V1 |
| Single monolithic crate | Compilation time | Only for initial scaffold |
| Hardcoded config paths | Breaks on other machines | Never |
| Skip agent rubric setup | Agent makes boundary violations | Only until .agents/ set up |

## Sources

- docs/blueprints/agent-native-starter-v1/01-north-star-and-principles.md
- docs/blueprints/agent-native-starter-v1/06-engineering-standards-rust-tauri-svelte.md
- docs/blueprints/agent-native-starter-v1/09-security-release-and-operations.md
- docs/blueprints/agent-native-starter-v1/11-rule-matrix-and-checklists.md

---
*Pitfalls research for: v0.2.0 架构蓝图对齐与核心功能实现*
*Researched: 2026-04-01*
