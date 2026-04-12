# Demo Counter E2E Tests

End-to-end tests for the counter demo flow. These tests verify the full-stack counter functionality including tenant isolation.

## Test Scenarios

1. Counter increment flow (single tenant)
2. Counter decrement flow
3. Counter reset flow
4. Multi-tenant counter isolation
5. Counter persistence across page reloads

## Running

```bash
# Via Justfile
just test-e2e-full

# Direct Playwright
npx playwright test verification/e2e/demo-counter/
```

## Architecture

These tests wrap the existing `apps/web/tests/e2e/counter.test.ts` and add orchestration
for full-stack validation (HTTP API → BFF → Service → Database).
