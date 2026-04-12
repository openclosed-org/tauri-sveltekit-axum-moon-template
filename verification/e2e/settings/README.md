# Settings E2E Tests

End-to-end tests for settings functionality (user-scoped settings).

## Test Scenarios

1. Settings CRUD flow (GET default → PUT update → GET updated)
2. User isolation (User A settings != User B settings)
3. Settings persistence across sessions
4. API key display with masking

## Running

```bash
npx playwright test verification/e2e/settings/
```
