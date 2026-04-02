# Phase 8: Desktop Native Features - Context

**Gathered:** 2026-03-29
**Status:** Ready for planning

<domain>
## Phase Boundary

App behaves as a polished desktop application with system tray, persistent window state, single-instance locking, and user-friendly error handling. Covers Tauri native feature integration — not business logic, not authentication, not database.

**Success Criteria (from ROADMAP.md):**
1. System tray icon appears with a menu offering Show/Hide/Quit options
2. Resizing and moving the window, then restarting, restores the exact previous position and size
3. Launching the app a second time focuses the existing window instead of opening a new one
4. Unhandled errors display a user-friendly toast/message dialog instead of a blank screen

**Not included:** Auth UI (Phase 6), database operations (Phase 5), cross-platform build verification (Phase 9).

</domain>

<decisions>
## Implementation Decisions

### System Tray (DESKTOP-01)
- **D-01:** Menu items: Show/Hide/Quit only — minimal menu, no separators, no quick actions
- **D-02:** Tray icon reuses existing `icons/32x32.png` from bundle assets — no new icon creation needed
- **D-03:** X button behavior: hide to tray (not quit). Quit only via tray menu. App keeps running in background
- **D-04:** Ctrl+Q keyboard shortcut also quits the app (standard desktop convention)

### Window State Persistence (DESKTOP-02)
- **D-05:** Persist: window size (width/height), position (x/y), and maximized state
- **D-06:** Minimum window size: 800×600 — prevents degenerate resize states
- **D-07:** Restore saved state during Tauri `setup()` callback — before window becomes visible, user sees correct size from first frame
- **D-08:** Use `tauri-plugin-window-state` (already declared in workspace deps + src-tauri/Cargo.toml, needs registration in lib.rs)

### Single Instance (DESKTOP-03)
- **D-09:** Use Tauri 2 built-in single instance mechanism — no additional plugin dependency
- **D-10:** On second launch: exit silently, focus existing window in first instance
- **D-11:** If existing window is hidden to tray: show it and bring to focus

### Error Handling (DESKTOP-04)
- **D-12:** Errors displayed as frontend toast notifications — non-blocking, auto-dismiss, top-right
- **D-13:** Error scope: Rust panics (global panic hook) + IPC command errors (already return Result<T, String>)
- **D-14:** Dismiss only — no retry buttons for boilerplate. Recovery logic left to boilerplate consumer
- **D-15:** Rust panic hook catches panics, emits a Tauri event to frontend, frontend shows toast

### Agent's Discretion
- Tray menu text (Show/Hide labels — use standard "Show" / "Hide" / "Quit" or localized)
- Whether Show/Hide is a single toggle item or two separate items
- Toast auto-dismiss timeout duration
- Panic hook's fallback message text (when panic payload isn't a string)
- Whether tray icon uses `tauri::tray::TrayIconBuilder` directly or a wrapper module

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase & Requirements
- `.planning/ROADMAP.md` §Phase 8 — Phase goal (desktop native features), success criteria (4 items), depends on Phase 4
- `.planning/REQUIREMENTS.md` §DESKTOP-01 through §DESKTOP-04 — Acceptance criteria for tray, window state, single instance, error handling

### Tauri 2 Documentation
- [Tauri 2 system tray](https://v2.tauri.app/features/system-tray/) — TrayIconBuilder, tray menu, click handlers
- [Tauri 2 window state plugin](https://github.com/tauri-apps/plugins-workspace/tree/v2/plugins/window-state) — register, state flags, save/restore API
- [Tauri 2 single instance](https://v2.tauri.app/features/single-instance/) — built-in single_instance() API, callback signature

### Prior Phase Context
- `.planning/phases/04-backend-dependencies-build-optimization/04-CONTEXT.md` — Cargo.toml preloaded `tauri-plugin-window-state` for Phase 8
- `.planning/phases/05-database-infrastructure/05-CONTEXT.md` — AppState pattern, tauri-plugin-libsql registration pattern (reference for plugin registration)
- `.planning/phases/07-multi-tenant-data-isolation/07-CONTEXT.md` — tauri-plugin-store session persistence pattern

### Existing Code
- `apps/desktop-ui/src-tauri/src/lib.rs` — Tauri builder entry point, plugin registration, setup() callback with auth timer
- `apps/desktop-ui/src-tauri/src/commands/auth.rs` — Auth commands, Result<T, String> error pattern, tauri-plugin-store usage
- `apps/desktop-ui/src-tauri/tauri.conf.json` — Window config (1200×800, resizable, icons array), deep-link config
- `apps/desktop-ui/src-tauri/Cargo.toml` — `tauri-plugin-window-state` already declared workspace=true
- `apps/desktop-ui/src-tauri/capabilities/default.json` — Current permissions (core, shell, dialog, store, deep-link)
- `Cargo.toml` — Workspace root, all Tauri plugins declared

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **tauri-plugin-window-state** — Already in workspace deps + src-tauri/Cargo.toml. Only needs `.plugin(tauri_plugin_window_state::Builder::default().build())` in lib.rs
- **tauri-plugin-dialog** — Already registered. Can be used for critical error dialogs if hybrid approach needed later
- **tauri-plugin-store** — Pattern established in auth.rs for persistent key-value storage (state flags could also use this)
- **Result<T, String> pattern** — All IPC commands return this. Frontend can intercept and display via toast
- **icons/32x32.png** — Already in bundle icon list, can be reused for tray icon

### Established Patterns
- Plugin registration in lib.rs: `.plugin(tauri_plugin_*::init())` or `Builder::default().build()`
- Setup callback in `tauri::Builder::default().setup(|app| { ... })` — auth timer already uses this
- Capabilities file needs explicit permissions for each plugin feature
- `AppHandle` used for cross-cutting concerns (store access, event emission)

### Integration Points
- `apps/desktop-ui/src-tauri/src/lib.rs` — Add window-state plugin registration, tray setup, single_instance callback, panic hook setup
- `apps/desktop-ui/src-tauri/capabilities/default.json` — Add tray, window-state, and core event permissions
- `tauri.conf.json` — May need `app.singleInstance` config or window minimum size
- Frontend toast component — SvelteKit side needs a toast listener for error events

</code_context>

<specifics>
## Specific Ideas

- Show/Hide as a single toggle: tray menu shows "Hide" when window is visible, "Show" when hidden
- Ctrl+Q as global quit shortcut (not just tray menu)
- Panic hook: `std::panic::set_hook()` that captures the panic message, emits `app.emit("error", { message })`, and lets the frontend toast component handle display
- Window state flags: use `WindowSizeState::all()` to save position + size + maximized
- Minimum size via `WindowBuilder::new().min_inner_size(800.0, 600.0)`

</specifics>

<deferred>
## Deferred Ideas

- Notification badges on tray icon (unread count) — v2 feature
- Tray quick actions (e.g., Login/Logout from tray) — beyond boilerplate scope
- Custom tray icon with platform-specific sizes (16×16 Windows, 22×22 Linux) — polish, not MVP
- Error retry with exponential backoff — v2 feature
- Error log persistence to file — v2 feature
- Multi-monitor-aware window restore — tauri-plugin-window-state handles this already

</deferred>

---

*Phase: 08-desktop-native-features*
*Context gathered: 2026-03-29*
