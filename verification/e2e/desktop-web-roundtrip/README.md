# Desktop + Web Roundtrip Tests

Tests that verify functionality across both Tauri desktop and web browser,
ensuring feature parity and consistent behavior.

## Test Scenarios

1. Counter works in both Tauri and Web
2. Settings work in both Tauri (commands) and Web (HTTP API)
3. Login/auth flow consistent
4. Tenant initialization consistent

## Running

This suite is outside the root backend-core contract.
If a derived project keeps the desktop and web shells, it should run this suite from shell-local tooling rather than root `just` commands.
