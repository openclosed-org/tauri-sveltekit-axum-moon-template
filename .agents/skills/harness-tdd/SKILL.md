---
name: harness-tdd
description: >
  Vertical-slice test-driven workflow for backend harness features and bug fixes.
  Use when adding or changing service, server, worker, contract, or repo-tool behavior,
  or when the user asks for TDD, red-green-refactor, or test-first implementation.
---

# Harness TDD

Use this skill to add or change behavior one vertical slice at a time.

Workflow skills guide process. Ownership skills still decide writable boundaries.

## Philosophy

Tests should verify observable behavior through public seams, not implementation details.

Prefer tests that survive internal refactors:

1. service public API or application seam
2. BFF handler or HTTP route behavior
3. worker replay, checkpoint, or projection seam
4. contract compatibility or drift seam
5. repo-tool CLI or validator seam

## Loop

For each behavior:

1. Pick one observable behavior.
2. Write one failing test at the correct public seam.
3. Run it and confirm RED.
4. Write the smallest correct code to pass.
5. Run it and confirm GREEN.
6. Refactor only while GREEN.
7. Run relevant path-scoped gates.

## Anti-Patterns

1. Do not write all tests first and then all implementation.
2. Do not test private helpers when a public seam can express the behavior.
3. Do not mock internal collaborators you own.
4. Do not add speculative behavior for future tests.
5. Do not expose internal seams only for test convenience.

## Mocking Rule

Mock true externals and hard runtime boundaries. Prefer local substitutes or in-memory adapters for owned seams when they provide better behavioral evidence.

## Refactor Check

After GREEN, look for shallow modules, duplicated rules, primitive obsession, hidden coupling, or oversized traits. If the fix reveals architecture friction, hand off to `harness-architecture-review`.
