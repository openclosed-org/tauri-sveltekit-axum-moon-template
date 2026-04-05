# Desktop E2E (Tauri WebDriver)

This directory contains desktop E2E tests for Tauri runtime behavior.

## Stack

- `tauri-driver` (WebDriver bridge for Tauri)
- `WebdriverIO` (`wdio`) with Mocha runner
- JUnit reporter for CI integration

## Test Coverage

**Total Specs: 5** (20+ test cases)

| Spec | Tests | Description |
|------|-------|-------------|
| `smoke.e2e.mjs` | 1 | Basic app shell loading |
| `login.e2e.mjs` | 4 | Login page, OAuth button, email input, responsive |
| `counter.e2e.mjs` | 4 | Auth guard, counter display, buttons, responsive |
| `admin.e2e.mjs` | 7 | Auth guard, dashboard layout, stat cards, responsive |
| `agent.e2e.mjs` | 7 | Auth guard, chat layout, sidebar, input, send button, responsive |

## Commands

From repo root:

```bash
# via web app script alias
bun run --cwd apps/client/web/app test:desktop

# direct invocation
bun run --cwd e2e-tests test

# CI mode with strict exit codes
bun run --cwd e2e-tests test:ci
```

## Platform behavior

- Linux / Windows: runs desktop E2E using `tauri-driver`.
- macOS: exits successfully with skip message by default because Tauri desktop WebDriver is not officially supported.
- Windows also needs a matching `msedgedriver.exe` on `PATH`, or `TAURI_NATIVE_DRIVER` pointing to it.
- You can force-run on unsupported platforms:

```bash
bun run --cwd e2e-tests test:force
```

## Test Results

After running tests, results are output to:

- **Console:** Real-time spec reporter
- **JUnit XML:** `test-results/junit/wdio-results-*.xml` (for CI)

## Notes

- The test runner builds the app in debug mode with `cargo tauri build --debug --no-bundle` before starting the WebDriver session.
- WDIO now explicitly opens `http://tauri.localhost` after the WebDriver session starts so Windows WebView2 does not stay on `about:blank`.
- Specs use `e2e-tests/helpers/navigate.mjs` with absolute app URL navigation to reduce cross-platform router flakiness.
- First-time setup requires `tauri-driver` in cargo bin path:

```bash
cargo install tauri-driver --locked
```

- On Windows CI or fresh local machines, install a matching Edge driver and expose it on `PATH`, or set `TAURI_NATIVE_DRIVER`.

- Tests verify auth guards, responsive design, and core functionality across all major pages.
- For detailed QA/UAT procedures, see `../qa-uat/README.md`
