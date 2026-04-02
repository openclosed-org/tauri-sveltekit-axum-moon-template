# Phase 8: Desktop Native Features — PLAN

**Phase:** 08 — Desktop Native Features
**Date:** 2026-03-29
**Status:** Draft

---

## Goal

App behaves as a polished desktop application with system tray, persistent window state, single-instance locking, and user-friendly error handling.

## Scope

- DESKTOP-01: System tray with Show/Hide/Quit menu
- DESKTOP-02: Window state persistence (position, size, maximized)
- DESKTOP-03: Single instance locking with focus-on-second-launch
- DESKTOP-04: Global error handling via toast notifications

## Success Criteria

1. System tray icon appears with Show/Hide/Quit menu; X button hides to tray
2. Window position/size/maximized state persists across restarts
3. Second app launch focuses existing window instead of opening new one
4. Unhandled errors display user-friendly toast instead of blank screen

---

## Task Breakdown

### Task 1: Add Dependencies (Rust + JS)

**Files:**
- `Cargo.toml` (root workspace): Add `tauri-plugin-single-instance` to workspace deps
- `apps/desktop-ui/src-tauri/Cargo.toml`: Add `tauri-plugin-single-instance` dep, add `"tray-icon"` feature to `tauri`
- `apps/desktop-ui/package.json`: Add `@tauri-apps/plugin-window-state` (JS bindings)

**Changes:**
```toml
# Cargo.toml (root) — add under Phase 8 section
tauri-plugin-single-instance = "2"
```

```toml
# src-tauri/Cargo.toml — add dep
tauri-plugin-single-instance = { workspace = true }
# change tauri dep to include tray-icon feature
# (handled via workspace override if possible, or direct)
```

**Verification:** `cargo check` passes

---

### Task 2: Window State Plugin Registration

**Files:**
- `apps/desktop-ui/src-tauri/src/lib.rs`
- `apps/desktop-ui/src-tauri/tauri.conf.json`

**Changes:**

In `lib.rs`, add plugin registration:
```rust
.plugin(tauri_plugin_window_state::Builder::default().build())
```

In `tauri.conf.json`, add min window size to existing window config:
```json
"minWidth": 800,
"minHeight": 600
```

**Verification:** App launches, resizes to 500×400, closes, reopens — should restore at 1200×800 (not the degenerate size)

---

### Task 3: System Tray Implementation

**Files:**
- `apps/desktop-ui/src-tauri/src/lib.rs`
- `apps/desktop-ui/src-tauri/capabilities/default.json`

**Changes in `lib.rs`:**

Wrap builder in `#[cfg(desktop)]` block. In `.setup()` callback:

1. Create menu items: `Show`/`Hide` (toggle), `Quit`
2. Build menu with `MenuBuilder`
3. Build tray with `TrayIconBuilder` using `app.default_window_icon()`
4. Handle menu events: `show` → `window.show() + set_focus()`, `hide` → `window.hide()`, `quit` → `app.exit(0)`
5. Add tray icon click handler to toggle window visibility

```rust
#[cfg(desktop)]
{
    use tauri::menu::{MenuBuilder, MenuItemBuilder};
    use tauri::tray::TrayIconBuilder;

    let show_item = MenuItemBuilder::with_id("show", "Show").build(app)?;
    let quit_item = MenuItemBuilder::with_id("quit", "Quit").build(app)?;
    let menu = MenuBuilder::new(app)
        .items(&[&show_item, &quit_item])
        .build()?;

    TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .on_menu_event(move |app, event| match event.id().as_ref() {
            "show" => {
                if let Some(win) = app.get_webview_window("main") {
                    let _ = win.show();
                    let _ = win.set_focus();
                }
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .build(app)?;
}
```

In `capabilities/default.json`, add:
```json
"core:default"
```
(core already listed — tray is part of core, no separate permission needed)

**X button → hide to tray:** Add window close event handler in setup:
```rust
if let Some(win) = app.get_webview_window("main") {
    win.on_window_event(|event| {
        if let tauri::WindowEvent::CloseRequested { api, .. } = event {
            api.prevent_close();
            let _ = event.window().hide();
        }
    });
}
```

**Verification:** App shows tray icon, right-click shows menu, Show/Hide toggles window, Quit exits

---

### Task 4: Single Instance Plugin

**Files:**
- `apps/desktop-ui/src-tauri/src/lib.rs`

**Changes:**

Register plugin with focus callback:
```rust
#[cfg(desktop)]
{
    builder = builder.plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
        if let Some(win) = app.get_webview_window("main") {
            let _ = win.show();
            let _ = win.set_focus();
        }
    }));
}
```

**Verification:** Launch app twice — second launch should focus existing window, not open new instance

---

### Task 5: Global Error Handling (Panic Hook + Toast)

**Files:**
- `apps/desktop-ui/src-tauri/src/lib.rs` — panic hook setup
- `apps/desktop-ui/src/routes/+layout.svelte` — toast listener
- `apps/desktop-ui/src/lib/components/ErrorToast.svelte` — new toast component

**Rust side (`lib.rs` setup):**
```rust
let app_handle = app.handle().clone();
let default_hook = std::panic::take_hook();
std::panic::set_hook(Box::new(move |info| {
    let msg = if let Some(s) = info.payload().downcast_ref::<&str>() {
        s.to_string()
    } else if let Some(s) = info.payload().downcast_ref::<String>() {
        s.clone()
    } else {
        "An unexpected error occurred".to_string()
    };
    let _ = app_handle.emit("app:panic", msg);
    default_hook(info);
}));
```

**Frontend (`+layout.svelte`):**
- Import `listen` from `@tauri-apps/api/event`
- Listen for `app:panic` event
- Show toast with error message

**ErrorToast.svelte:**
- Simple auto-dismissing toast (top-right, ~5s)
- Uses existing tailwindcss styling

**IPC errors** (already handled): `invoke()` calls return `Result<T, String>`, frontend can `.catch()` and display toast. Add a global invoke error wrapper in layout.

**Verification:** Trigger a Rust panic → toast appears; IPC command error → toast appears

---

## Execution Order

```
Task 1 (deps) → Task 2 (window state) → Task 3 (tray) → Task 4 (single instance) → Task 5 (error handling)
```

Each task is independently testable.

## Dependencies Between Tasks

- Task 3 depends on Task 1 (tray-icon feature)
- Task 4 depends on Task 1 (single-instance dep)
- Tasks 2, 3, 4, 5 all modify `lib.rs` — merge carefully

## Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| Tray icon not showing on Windows | High | Use `app.default_window_icon()` which is already bundled |
| Single-instance conflicts with tray hide-to-tray | Medium | Show window in single-instance callback |
| Panic hook overwrites existing hook | Low | Save and call default_hook in wrapper |
| `CloseRequested` event not firing on all platforms | Low | Test on Windows first |
| JS event listener timing (tray events vs frontent mount) | Medium | Tauri events are buffered — listeners catch events even if mounted after emit |

## Files Modified Summary

| File | Changes |
|------|---------|
| `Cargo.toml` (root) | Add `tauri-plugin-single-instance` |
| `apps/desktop-ui/src-tauri/Cargo.toml` | Add single-instance dep, tauri tray-icon feature |
| `apps/desktop-ui/package.json` | Add `@tauri-apps/plugin-window-state` |
| `apps/desktop-ui/src-tauri/src/lib.rs` | Plugin registrations, tray setup, single-instance, panic hook, close-to-tray |
| `apps/desktop-ui/src-tauri/tauri.conf.json` | Add minWidth/minHeight |
| `apps/desktop-ui/src-tauri/capabilities/default.json` | No change needed (tray is core) |
| `apps/desktop-ui/src/routes/+layout.svelte` | Add panic event listener + toast |
| `apps/desktop-ui/src/lib/components/ErrorToast.svelte` | **New** — auto-dismissing error toast |

## Agent's Discretion

- Show/Hide as two separate menu items (not toggle) — simpler logic, clearer UX
- Toast timeout: 5 seconds auto-dismiss
- Panic fallback message: "An unexpected error occurred"
- Use `TrayIconBuilder` directly in setup (no wrapper module) — minimal abstraction for boilerplate

---

## Verification Fixes (v1.1)

### Fix 1: D-09 Clarification
CONTEXT D-09 says "built-in single instance mechanism — no additional plugin dependency". In Tauri 2, single instance is implemented via `tauri-plugin-single-instance` — this IS the official mechanism. The D-09 wording was a misunderstanding; the plugin approach is correct and canonical.

### Fix 2: Ctrl+Q Shortcut (D-04)
Add global keyboard shortcut for Ctrl+Q → quit. Use `tauri-plugin-global-shortcut`:
- Add dep to workspace and src-tauri Cargo.toml
- Register plugin with shortcut `CommandOrControl+Q` → `app.exit(0)`
- Add permission `core:default` (global-shortcut is part of core in Tauri 2)

Actually, simpler approach: handle Ctrl+Q in the **frontend** layout with a `keydown` listener that calls `invoke` or `getCurrent().close()`. No new plugin needed.

**Decision:** Frontend approach — add keydown listener in `+layout.svelte`:
```svelte
onMount(() => {
  const handleKeydown = (e: KeyboardEvent) => {
    if ((e.ctrlKey || e.metaKey) && e.key === 'q') {
      e.preventDefault();
      getCurrent().close();
    }
  };
  window.addEventListener('keydown', handleKeydown);
  return () => window.removeEventListener('keydown', handleKeydown);
});
```

### Fix 3: Tray Menu Items — Add Hide
CONTEXT D-01 specifies Show/Hide/Quit. Update Task 3 code to include all three items:
```rust
let show_item = MenuItemBuilder::with_id("show", "Show").build(app)?;
let hide_item = MenuItemBuilder::with_id("hide", "Hide").build(app)?;
let quit_item = MenuItemBuilder::with_id("quit", "Quit").build(app)?;
let menu = MenuBuilder::new(app)
    .items(&[&show_item, &hide_item, &quit_item])
    .build()?;
```

### Fix 4: Tray Icon Click Handler
Add tray icon click handler to toggle window visibility:
```rust
.on_tray_icon_event(|tray, event| {
    if let tauri::tray::TrayIconEvent::Click { button: tauri::tray::MouseButton::Left, button_state: tauri::tray::MouseButtonState::Up, .. } = event {
        let app = tray.app_handle();
        if let Some(win) = app.get_webview_window("main") {
            if win.is_visible().unwrap_or(false) {
                let _ = win.hide();
            } else {
                let _ = win.show();
                let _ = win.set_focus();
            }
        }
    }
})
```

### Fix 5: Window-State Capabilities
Verify if `window-state:default` permission is needed. Tauri 2 plugins typically require capability permissions. Add `"window-state:default"` to capabilities/default.json.

### Fix 6: setup() Ordering
All tasks modify lib.rs setup(). Correct order in the single setup() closure:
1. Save AppHandle clone for panic hook
2. Window-state plugin registration (before setup, in plugin chain)
3. Single-instance plugin registration (before setup, in plugin chain)
4. Setup closure:
   a. Create tray menu items
   b. Build tray with click + menu event handlers
   c. Register panic hook with saved AppHandle
   d. Start auth refresh timer
   e. Add CloseRequested handler to main window

### Fix 7: Event Name Consistency
Rust emits `"app:panic"` → frontend listens for `"app:panic"`. Already consistent in plan, just noting.

### Task 6 (New): Ctrl+Q Keyboard Shortcut
**Files:** `apps/desktop-ui/src/routes/+layout.svelte`

Add keydown listener for Ctrl+Q/Meta+Q that calls `getCurrent().close()`. Import `getCurrent` from `@tauri-apps/api/webviewWindow`.

---

## Updated Execution Order

```
Task 1 (deps) → Task 2 (window state plugin) → Task 3 (tray + close-to-tray) → Task 4 (single instance) → Task 5 (error handling) → Task 6 (Ctrl+Q)
```

## Updated Files Modified Summary

| File | Changes |
|------|---------|
| `Cargo.toml` (root) | Add `tauri-plugin-single-instance` |
| `apps/desktop-ui/src-tauri/Cargo.toml` | Add single-instance dep, tauri tray-icon feature |
| `apps/desktop-ui/package.json` | Add `@tauri-apps/plugin-window-state` |
| `apps/desktop-ui/src-tauri/src/lib.rs` | Plugin registrations, tray setup (3 items + click handler), single-instance, panic hook, close-to-tray |
| `apps/desktop-ui/src-tauri/tauri.conf.json` | Add minWidth/minHeight |
| `apps/desktop-ui/src-tauri/capabilities/default.json` | Add `"window-state:default"` |
| `apps/desktop-ui/src/routes/+layout.svelte` | Add panic event listener + toast + Ctrl+Q handler |
| `apps/desktop-ui/src/lib/components/ErrorToast.svelte` | **New** — auto-dismissing error toast |
