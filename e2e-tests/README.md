# Desktop E2E (Tauri WebDriver)

This directory contains desktop E2E tests for Tauri runtime behavior.

## Stack

- `tauri-driver` (WebDriver bridge for Tauri)
- `WebdriverIO` (`wdio`) with Mocha runner

## Commands

From repo root:

```bash
# via web app script alias
bun run --cwd apps/client/web/app test:desktop

# direct invocation
bun run --cwd e2e-tests test
```

## Platform behavior

- Linux / Windows: runs desktop E2E using `tauri-driver`.
- macOS: exits successfully with skip message by default because Tauri desktop WebDriver is not officially supported.
- You can force-run on unsupported platforms:

```bash
bun run --cwd e2e-tests test:force
```

## Notes

- The test runner builds the app in debug mode with `cargo tauri build --debug --no-bundle` before starting the WebDriver session.
- First-time setup requires `tauri-driver` in cargo bin path:

```bash
cargo install tauri-driver --locked
```
