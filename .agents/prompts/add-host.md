# Add Host Prompt

**Purpose:** Guide an agent through adding a new Tauri host adapter, bridging frontend IPC calls to backend usecases.
**When to Use:** When you need a new Tauri command that exposes existing or new usecase functionality to the SvelteKit frontend.

---

## Prerequisites

- [ ] Read `AGENTS.md` for execution protocol
- [ ] Read `.agents/rubrics/boundary-compliance.md` — host layer import rules
- [ ] Confirm the usecase service trait already exists in `packages/core/usecases/`
- [ ] Run `just verify` to confirm current state is clean

---

## Steps

### 1. Create Tauri Command Handler

Follow the `counter.rs` pattern in `packages/adapters/hosts/tauri/src/commands/`:

```rust
#[tauri::command]
pub async fn my_command(
    app: tauri::AppHandle,
    input: MyCommandInput,
) -> Result<MyCommandOutput, String> {
    let state = app.state::<AppState>();
    let service = state.my_service.clone();
    service.execute(input).await.map_err(|e| e.to_string())
}
```

Key rules:
- Command receives `AppHandle` to access application state
- Command delegates to usecases service — **no inline business logic**
- Command maps between Tauri-serializable types and usecases types
- Errors are converted to `String` for Tauri IPC compatibility

### 2. Register Command in Module

Add the new command to `packages/adapters/hosts/tauri/src/commands/mod.rs`:

```rust
pub mod my_command;
pub use my_command::my_command;
```

### 3. Register in Invoke Handler

Add the command to the `invoke_handler` in `packages/adapters/hosts/tauri/src/lib.rs`:

```rust
.invoke_handler(tauri::generate_handler![
    // existing commands...
    my_command,
])
```

### 4. Add Cargo Dependencies (if needed)

If the command needs new crates, add them to `packages/adapters/hosts/tauri/Cargo.toml`. Avoid introducing new dependencies unless necessary.

### 5. Frontend IPC Client

Create or update the frontend IPC client in `frontend/src/lib/ipc/`:

```typescript
import { invoke } from '@tauri-apps/api/core'

export async function myCommand(input: MyCommandInput): Promise<MyCommandOutput> {
    return await invoke('my_command', { input })
}
```

---

## Verification

```bash
# Full quality check
just verify

# Rust tests
cargo test
```

Manual checks:
- [ ] Command delegates to usecases, no business logic inline
- [ ] Command registered in `mod.rs` and `invoke_handler`
- [ ] No `unwrap()` in command handler
- [ ] Error messages don't leak sensitive information
- [ ] Frontend IPC client uses generated types from `frontend/generated/`

---

## Boundary Compliance

Per `.agents/rubrics/boundary-compliance.md`:

- [ ] Host adapter CAN import: domain, usecases
- [ ] Host adapter MUST NOT contain business rules
- [ ] Command handlers delegate to usecases service trait
- [ ] AppState accessed via `app.state::<T>()`, not global singletons

---

## References

- `.agents/playbooks/create-feature.md` — Step 5 (Host + Frontend)
- `.agents/rubrics/boundary-compliance.md` — Host layer rules
- `packages/adapters/hosts/tauri/src/commands/counter.rs` — Reference implementation
