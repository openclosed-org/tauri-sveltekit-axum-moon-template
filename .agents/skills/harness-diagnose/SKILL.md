---
name: harness-diagnose
description: >
  Disciplined diagnosis workflow for failing gates, regressions, flaky behavior, performance regressions,
  and suspicious backend/runtime behavior. Use when the user reports a bug, broken invariant,
  failing command, failing test, or asks to diagnose/debug a problem.
---

# Harness Diagnose

Use this skill for bugs and regressions.

Workflow skills guide process. Ownership skills still decide writable boundaries.

## Phases

1. Build a feedback loop.
2. Reproduce the exact symptom.
3. Generate 3-5 ranked falsifiable hypotheses.
4. Instrument one boundary or variable at a time.
5. Fix the smallest causal closed loop.
6. Add regression evidence at the same semantic level.
7. Run path-scoped gates from `agent/manifests/gate-matrix.yml`.
8. Report evidence and residual uncertainty.

## Feedback Loop Options

Prefer fast, deterministic, agent-runnable signals:

1. failing unit, service, integration, replay, or contract test
2. validator or gate command
3. CLI invocation with fixture input and expected output
4. HTTP request against a running server
5. replayed event, outbox, or projector trace
6. throwaway harness isolated to one service, server, worker, or repo-tool path
7. stress loop for flakes or concurrency issues

If no credible loop can be built, stop and state what was tried and what artifact is needed.

## Instrumentation Rules

1. Every probe must test one hypothesis.
2. Prefer debugger, targeted assertion, or narrow logs over broad logging.
3. Tag temporary logs with a unique prefix and remove them before completion.
4. For performance regressions, measure first and fix second.

## Completion Report

State:

1. violated invariant
2. causal boundary: contract, service, server, worker, platform, tooling, docs, or cross-boundary
3. repair made
4. regression evidence added or why no correct seam exists
5. gates run
6. gates not run
7. residual risk or uncertainty
