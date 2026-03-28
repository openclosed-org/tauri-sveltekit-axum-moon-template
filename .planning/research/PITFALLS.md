# Pitfalls Research

**Domain:** Tauri+SvelteKit+Axum Full-Stack Desktop Application
**Researched:** 2026-03-28
**Confidence:** MEDIUM-HIGH

---

## Critical Pitfalls

### Pitfall 1: Tauri 2 Permission System Misconfiguration

**What goes wrong:**
Application fails at runtime with "Permission denied" errors. Features like file system access, HTTP requests, or clipboard operations silently fail or throw cryptic errors.

**Why it happens:**
Tauri 2 uses a "Deny by Default" security model. Unlike Tauri 1 or Electron, all system access is blocked unless explicitly permitted via Capabilities. Developers assume plugins work out-of-the-box after installation.

**How to avoid:**
1. Configure capabilities in `src-tauri/capabilities/` directory
2. Explicitly declare permissions for each plugin used:

```json
{
  "identifier": "main-capability",
  "description": "Main window permissions",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "fs:default",
    "fs:allow-read-text-file",
    "http:default"
  ]
}
```

3. For scoped permissions (like file system), specify exact paths:

```json
"fs:allow-read-file",
{
  "identifier": "fs:scope",
  "allow": [
    { "path": "$APPDATA/*" },
    { "path": "$DOCUMENT/*" }
  ]
}
```

**Warning signs:**
- Plugin installed via `cargo add` or npm but no capability file updated
- "invoke() function not found" errors in console
- Application works in dev mode but fails in production build

**Phase to address:** Foundation Phase (Phase 1) — must configure capabilities before any feature development

---

### Pitfall 2: Bundle Size Bloat Due to Unoptimized Cargo Config

**What goes wrong:**
Tauri application produces 50MB+ executables when 5-10MB should be achievable. Users resist downloading, CI/CD pipelines slow down.

**Why it happens:**
Default Cargo build settings prioritize compile speed over binary size. Developers don't configure release profiles or enable link-time optimization (LTO).

**How to avoid:**
Configure release profile in `src-tauri/Cargo.toml`:

```toml
[profile.release]
codegen-units = 1
lto = true
opt-level = "z"
panic = "abort"
strip = true
```

Additionally, enable `removeUnusedCommands` in `tauri.conf.json`:

```json
{
  "build": {
    "removeUnusedCommands": true
  }
}
```

This automatically strips Tauri commands not exposed in your capability files.

**Warning signs:**
- Release build directory exceeds 30MB
- No `[profile.release]` section in Cargo.toml
- Using debug build for "testing performance"

**Phase to address:** Foundation Phase (Phase 1) — set up optimized build config from start

---

### Pitfall 3: IPC vs HTTP Performance Mismatch

**What goes wrong:**
Application makes unnecessary HTTP calls to Axum backend running on localhost, adding 50-200ms latency per request when direct IPC would take 1-5ms.

**Why it happens:**
Confusion between "local API" (Axum on localhost) and "remote API" (external service). Using HTTP for frontend-to-backend communication within the same application adds WebView network stack overhead.

**How to avoid:**
- **Use Tauri IPC (`invoke()`)** for all Axum backend communication because:
  - Same machine (localhost) — no network benefits
  - IPC bypasses HTTP parsing overhead
  - Type-safe across Rust/TypeScript boundary
  
- **Use HTTP plugin (`fetch()`)** only for:
  - Actual external APIs (payment gateways, OAuth, third-party services)
  - Situations where HTTP is required by external service

```typescript
// ❌ Wrong: HTTP to local Axum
const response = await fetch('http://localhost:1420/api/data');
const data = await response.json();

// ✅ Correct: Tauri IPC invoke
const data = await invoke<{ id: number; name: string }[]>('get_data');
```

**Warning signs:**
- Frontend makes `fetch()` calls to `localhost` or `127.0.0.1`
- Network tab shows "HTTP/1.1 200" for local addresses
- API response times exceed 100ms for simple queries

**Phase to address:** Integration Phase (Phase 2) — establish correct communication patterns early

---

### Pitfall 4: Database Migration Missing in Production

**What goes wrong:**
SQLite database fails to load on first production run, or schema is outdated causing runtime errors. Works perfectly in development but crashes for users.

**Why it happens:**
- Migrations embedded at compile time fail to find the database file in production's app data directory
- No migration runner executes on startup
- Assumes database file is copied with application bundle

**How to avoid:**
1. Use proper migration library like `rusqlite_migration`:

```rust
use rusqlite_migration::{Engine, M};

fn main() {
    let migrations = vec![
        M::up("CREATE TABLE user (id INTEGER PRIMARY KEY, name TEXT)"),
        M::up("ALTER TABLE user ADD COLUMN email TEXT"),
    ];
    
    let db_path = get_app_data_dir().join("app.db");
    let mut engine = Engine::builder(db_path)
        .migrations(&migrations)
        .build();
    
    // Runs migrations automatically if needed
    engine.execute_migrations();
}
```

2. Store database in OS-appropriate location (use `tauri::api::path` or `dirs` crate):

```rust
use dirs::data_dir;
let app_data = data_dir().unwrap().join("your-app");
std::fs::create_dir_all(&app_data).ok();
let db_path = app_data.join("app.db");
```

3. Initialize migrations at application startup, not just first run

**Warning signs:**
- Database path hardcoded to project directory
- No migration logic visible in startup code
- Database file exists in source directory but not in installed app

**Phase to address:** Data Layer Phase (Phase 3) — implement migrations with proper path handling

---

### Pitfall 5: Desktop Security Model Misunderstanding

**What goes wrong:**
Application has security vulnerabilities exposing user data or allowing remote code execution through malicious web content.

**Why it happens:**
- Assuming WebView is same as browser — it's not sandboxed by default
- Using `nodeIntegration: true` patterns from Electron without understanding Tauri differences
- Exposing backend commands without input validation

**How to avoid:**
1. **Never trust frontend input** — validate everything in Rust handlers:

```rust
#[tauri::command]
fn process_user_input(input: String) -> Result<String, String> {
    // Validate length, content, SQL injection attempts
    if input.len() > 1000 {
        return Err("Input too long".into());
    }
    if input.contains("DROP TABLE") || input.contains("--") {
        return Err("Suspicious content".into());
    }
    // Proceed with sanitized input
}
```

2. **Configure Content Security Policy (CSP)** in `tauri.conf.json`:

```json
{
  "security": {
    "csp": "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'"
  }
}
```

3. **Use capabilities, not broad permissions** — avoid `allow: *`:

```json
// ❌ Too broad
"fs:allow-read-file"

// ✅ Specific scope
"fs:allow-read-file",
{
  "identifier": "fs:scope",
  "allow": [{ "path": "$APPDATA/my-app/*" }]
}
```

4. **Never embed secrets in binary** — use environment variables at runtime, not build time

**Warning signs:**
- No CSP configured in tauri.conf.json
- Using `allow: *` in any capability
- Backend commands接受原始字符串输入 without validation
- .env files included in production bundle

**Phase to address:** Foundation Phase (Phase 1) — security model must be correct from start

---

### Pitfall 6: Cross-Platform Build Complexity Underestimated

**What goes wrong:**
- Windows build fails when generated on macOS (or vice versa)
- WebView2 runtime missing on Windows target machines
- Platform-specific entitlements block macOS builds
- Path separators break on different OS

**Why it happens:**
- Assuming "one build works everywhere" without cross-compilation setup
- Not testing on target platform before release
- Hardcoded paths using `/` or `\` instead of Rust's `Path` or `std::path::MAIN_SEPARATOR`

**How to avoid:**
1. **Use `cross` tool or CI/CD for cross-compilation** — requires:
   - `x86_64-pc-windows-gnu` target on macOS/Linux
   - Appropriate Windows SDK

2. **Handle paths correctly in Rust:**

```rust
use std::path::PathBuf;

fn get_config_path() -> PathBuf {
    let mut path = dirs::config_dir().unwrap();
    path.push("my-app");
    std::fs::create_dir_all(&path).ok();
    path.push("config.json");
    path
}

// In frontend (TypeScript), use Tauri path APIs:
import { path } from '@tauri-apps/api';
const configPath = await path.configDir();
```

3. **Configure WebView2 bootstrap for Windows** in NSIS/MSI installer:
   - Set `installerHints` or bundle WebView2installer

4. **Test on target platform before release** — at minimum, build on all target platforms

**Warning signs:**
- Build uses only local OS toolchain
- Path strings contain hardcoded separators
- No WebView2 fallback logic in Windows build
- `panic = "abort"` not set (causes larger binaries with panic data)

**Phase to address:** Build & Distribution Phase (Phase 4) — implement proper cross-platform build pipeline

---

## Technical Debt Patterns

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|----------------|-----------------|
| Skipping capability config in dev | "Works in dev mode" | Breaks in production with cryptic errors | Never — capabilities required from start |
| Using HTTP for local IPC | "Consistent with web patterns" | 100ms+ latency per call | Never — use invoke() for local |
| Hardcoding database path | Simple local development | Fails on different OS installs | Never — use OS API paths |
| Bypassing validation in IPC | "Faster development" | Security vulnerabilities | Never |
| Debug build for performance testing | Faster compiles | Misleading benchmarks | Never — use release for any performance work |

---

## Integration Gotchas

| Integration | Common Mistake | Correct Approach |
|-------------|----------------|------------------|
| Axum HTTP server | Using `127.0.0.1` which binds to specific interface | Use `0.0.0.0` for desktop, but prefer IPC for local communication |
| SQLite database | Not creating app data directory | Ensure `$APPDATA/app-name/` exists before DB operations |
| Tauri plugin initialization | Initializing plugins after Tauri builder | Add plugins in `tauri::Builder::default().plugin(...)` chain |
| Environment variables | Embedding secrets at build time | Load from `.env` at runtime, use keychain plugins for production |
| WebView2 (Windows) | Assuming it's always installed | Bundle installer or check at runtime with graceful fallback |

---

## Performance Traps

| Trap | Symptoms | Prevention | When It Breaks |
|------|----------|------------|----------------|
| HTTP to localhost | 100ms+ API latency | Use Tauri `invoke()` | Every request — significant cumulative delay |
| Large IPC payloads | UI freezes during data transfer | Chunk data >1MB, use streaming | Large database queries, file operations |
| No binary size optimization | 50MB+ executables | Configure LTO, strip, opt-level=z | Distribution — users refuse downloads |
| Blocking IPC on main thread | UI freezes | Use `tokio::spawn` for long operations | Any blocking operation >100ms |
| Memory leaks in Rust | Growing memory usage | Use appropriate ownership patterns | Long-running applications |

---

## Security Mistakes

| Mistake | Risk | Prevention |
|---------|------|------------|
| No CSP configured | XSS attacks execute malicious scripts | Set strict CSP in tauri.conf.json |
| Broad capability permissions | Compromise allows wider system access | Use specific permissions with scopes |
| No input validation in IPC | SQL injection, command injection | Validate all arguments in Rust handlers |
| Embedding secrets in binary | Reverse engineering exposes credentials | Use runtime env vars, keychain plugins |
| `nodeIntegration` equivalent | Not applicable to Tauri (different model) | Never enable Node.js in WebView |
| No capability isolation | One compromised feature exposes all | Use separate capabilities per feature window |

---

## UX Pitfalls

| Pitfall | User Impact | Better Approach |
|---------|-------------|-----------------|
| No loading states for IPC | User thinks app is frozen | Show loading indicators during invoke() calls |
| Blank window on production, fine in dev | User can't use app | Test production build before release; check WebView availability |
| No error messages for permission denials | Users don't know why features fail | Display friendly error dialogs with resolution steps |
| Different behavior dev vs prod | QA misses critical bugs | Mirror production config in development |

---

## "Looks Done But Isn't" Checklist

- [ ] **Capabilities:** Configured but not tested in production build — verify on fresh install
- [ ] **Database migrations:** Tested in dev but not in clean production environment
- [ ] **IPC validation:** Backend accepts calls but never validated malicious input
- [ ] **CSP:** Configured but allows `'unsafe-inline'` — cookie-monster attacks possible
- [ ] **File paths:** Work in source directory but fail in installed app location
- [ ] **Cross-platform builds:** Windows build tested on Windows before release
- [ ] **WebView2:** Windows deployment includes fallback for missing runtime
- [ ] **Error handling:** IPC errors caught but no user-facing messages

---

## Recovery Strategies

| Pitfall | Recovery Cost | Recovery Steps |
|---------|---------------|----------------|
| Permission denied in production | MEDIUM | Add missing capability to JSON file, rebuild and release |
| Database migration failed | HIGH | Implement backup/restore, manual migration path for users |
| Security vulnerability found | HIGH | Patch release, potential user notification, audit logs |
| Cross-platform build failure | MEDIUM | Set up proper CI/CD with target platform toolchains |
| Bundle size too large | LOW | Add optimize flags, rebuild — typically 50%+ reduction |

---

## Pitfall-to-Phase Mapping

| Pitfall | Prevention Phase | Verification |
|---------|------------------|--------------|
| Permission system misconfiguration | Foundation (Phase 1) | Build production .exe, test in clean environment |
| Bundle size bloat | Foundation (Phase 1) | Check release binary size <20MB |
| IPC vs HTTP mismatch | Integration (Phase 2) | Network tab shows no localhost HTTP calls |
| Database migration issues | Data Layer (Phase 3) | Fresh install on clean system works |
| Security model gaps | Foundation (Phase 1) | Security audit, CSP testing |
| Cross-platform builds | Build & Distribution (Phase 4) | Build on all target platforms, test each |

---

## Sources

- Tauri v2 Documentation: https://tauri.app (App Size, Security, Permissions sections)
- Tauri v2 Security Model Guide: https://www.oflight.co.jp/en/columns/tauri-v2-security-model (Comprehensive permission-based access control)
- Tauri GitHub Issues: #14259 (fs permissions bug), #12312 (cross-platform compilation)
- Community Discussions: Tauri Discord, Reddit r/tauri
- Rust crates: rusqlite_migration, tauri-plugin-fs, tauri-plugin-http
- CVE Database comparison (Electron vs Tauri security)

---

*Pitfalls research for: Tauri+SvelteKit+Axum Full-Stack Desktop Application*
*Researched: 2026-03-28*