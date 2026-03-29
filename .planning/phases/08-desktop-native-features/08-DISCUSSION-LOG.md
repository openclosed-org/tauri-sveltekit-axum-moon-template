# Phase 8: Desktop Native Features - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-03-29
**Phase:** 08-desktop-native-features
**Areas discussed:** Tray Icon Behavior, Window State Granularity, Single Instance Strategy, Error Handling UX

---

## Tray Icon Behavior

| Option | Description | Selected |
|--------|-------------|----------|
| Show/Hide/Quit only | Minimal menu matching the requirement | ✓ |
| Show/Hide/Quit + separator | Same items with visual separator | |
| Full menu + quick actions | Show/Hide, Login/Status, Settings, Quit | |

### Close Behavior

| Option | Description | Selected |
|--------|-------------|----------|
| Hide to tray | X hides window, app keeps running | ✓ |
| Ask on first close | Dialog asks, save preference | |
| Quit on close | X closes app entirely | |

### Tray Icon Source

| Option | Description | Selected |
|--------|-------------|----------|
| Reuse existing icons | Use 32x32.png from bundle icons | ✓ |
| Custom tray-specific icon | Separate 16x16 or 22x22 icon | |
| Native platform icon | Use OS-provided icon | |

**User's choice:** Show/Hide/Quit only, hide to tray, reuse existing 32x32.png
**Notes:** Minimal tray menu. Ctrl+Q also quits.

---

## Window State Granularity

| Option | Description | Selected |
|--------|-------------|----------|
| Size + position + maximized | Save width, height, x, y, maximized state | ✓ |
| Size + position only | Only dimensions and screen position | |
| Full state including display | Size + position + maximized + which monitor | |

### Size Constraints

| Option | Description | Selected |
|--------|-------------|----------|
| 800x600 minimum | Prevents degenerate resize states | ✓ |
| 400x300 minimum | More permissive minimum | |
| No minimum | Free resize, no constraints | |

### Restore Timing

| Option | Description | Selected |
|--------|-------------|----------|
| During Tauri setup | Restore before window visible | ✓ |
| After setup, before show | Initialize, apply state, then show | |
| Lazy restore | Show at default, animate to saved | |

**User's choice:** Size + position + maximized, 800x600 minimum, restore during setup()
**Notes:** tauri-plugin-window-state already declared, needs registration.

---

## Single Instance Strategy

| Option | Description | Selected |
|--------|-------------|----------|
| Tauri 2 built-in | single_instance() API, no plugin | ✓ |
| tauri-plugin-single-instance | Community plugin | |
| Manual OS-level lock | Lock file or OS mutex | |

### Second Launch Behavior

| Option | Description | Selected |
|--------|-------------|----------|
| Focus existing window | Exit second, show first | ✓ |
| Focus + pass arguments | Pass CLI args to first instance | |
| Notify existing | First instance gets event | |

### Hidden Window Handling

| Option | Description | Selected |
|--------|-------------|----------|
| Yes, show from tray | Show + focus existing window | ✓ |
| No, stay hidden | Respect user's hide choice | |

**User's choice:** Tauri 2 built-in, focus existing window, show from tray if hidden
**Notes:** No plugin dependency. Second launch exits silently.

---

## Error Handling UX

| Option | Description | Selected |
|--------|-------------|----------|
| Frontend toast notification | Non-blocking, auto-dismiss | ✓ |
| OS-native dialog | Blocking message dialog | |
| Hybrid: toast + dialog | Non-critical → toast, critical → dialog | |

### Error Scope

| Option | Description | Selected |
|--------|-------------|----------|
| Rust panics + IPC errors | Global panic hook + Result<T,String> | ✓ |
| Everything including frontend | Panics + IPC + JS errors | |
| IPC errors only | Only Tauri command failures | |

### Error Recovery

| Option | Description | Selected |
|--------|-------------|----------|
| Dismiss only | Toast with close button | ✓ |
| Retry button on network errors | Retry alongside message | |
| Log + dismiss | Auto-dismiss + persist to log | |

**User's choice:** Frontend toast, Rust panics + IPC errors, dismiss only
**Notes:** Panic hook emits Tauri event. Frontend toast component listens. No retry for boilerplate.

---

## Agent's Discretion

- Toast auto-dismiss timeout
- Panic hook fallback message text
- Whether tray menu uses toggle (Hide/Show) or two items
- Exact TrayIconBuilder API usage vs wrapper module

---

## Deferred Ideas

- Notification badges on tray icon
- Tray quick actions (Login/Logout)
- Custom tray icon with platform-specific sizes
- Error retry with backoff
- Error log persistence
