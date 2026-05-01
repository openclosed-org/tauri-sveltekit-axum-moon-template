# Architecture Deepening Language

This file defines vocabulary for architecture reviews and refactoring discussions. It is vocabulary, not a mandate to refactor.

## Terms

**Module**
Anything with an interface and implementation: a function, type, package, service library, worker, server, or vertical slice.
_Avoid_: component when the concern is interface depth.

**Interface**
Everything a caller must know to use a module correctly: types, invariants, ordering, error modes, configuration, and performance assumptions.
_Avoid_: signature when the concern is broader than type shape.

**Implementation**
The code and behavior hidden behind a module interface.
_Avoid_: adapter when the concern is not a seam implementation.

**Seam**
A place where behavior can be altered without editing the caller in place.
_Avoid_: boundary when the concern is not a DDD or ownership boundary.

**Adapter**
A concrete implementation that satisfies an interface at a seam.
_Avoid_: module when the role at a seam matters.

**Depth**
The amount of useful behavior behind a small, stable interface.
_Avoid_: line-count ratio.

**Leverage**
What callers gain when one interface exposes substantial behavior without requiring them to know internal details.
_Avoid_: convenience wrapper.

**Locality**
What maintainers gain when change, bugs, and verification concentrate behind one interface instead of spreading across callers.
_Avoid_: scattering.

**Deletion test**
A review question: if this module disappeared, would complexity vanish, or would it reappear across multiple callers?
_Avoid_: abstract purity test.

## Principles

1. The interface is the test surface.
2. One adapter usually means a hypothetical seam; two justified adapters make the seam more real.
3. Do not expose internal seams only to make tests easier.
4. A service library can be a semantic boundary without being a runtime process boundary.
5. A Rust trait is not automatically a useful seam; it needs a real variation, testing, or topology-late story.
6. Deepening should improve leverage, locality, or testability without violating ownership boundaries.

## Dependency Categories

1. **In-process**: pure or local behavior; usually deepenable behind one interface.
2. **Local-substitutable**: dependencies with local test stand-ins; test through the deepened interface using the stand-in.
3. **Remote but owned**: internal service or worker dependency; use a port only when runtime and test adapters are justified.
4. **True external**: third-party dependency; inject a narrow port and test with a mock adapter.
