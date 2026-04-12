# Multi-Tenant E2E Tests

End-to-end tests verifying tenant isolation and multi-tenant flows.

## Test Scenarios

1. Tenant A cannot see Tenant B's counters
2. Tenant onboarding flow (init → create → verify)
3. Cross-tenant API returns 403/404
4. Tenant member management

## Running

```bash
npx playwright test verification/e2e/multi-tenant/
```
