# ADR-008: Wasm Extension Plane

## Status
- [x] Proposed
- [ ] Accepted
- [ ] Deprecated
- [ ] Superseded

> **Implementation Status**: All Wasm components are deferred — WIT definitions, host runtime,
> and guest SDK are not yet implemented. This ADR describes a future capability, not current reality.
> It is not part of the current default agent path.

## Context
The system needs extensibility mechanisms for:
- Custom business rules per tenant
- Protocol transformations (Nostr, Farcaster, AT Protocol, etc.)
- Policy hooks (timeout, retry, authz overrides)
- Plugin-based sync strategies
- Third-party integrations without core code changes

Traditional approaches (shared libraries, subprocesses, HTTP callbacks) have limitations:
- **Shared libraries**: Require recompilation, tight coupling
- **Subprocesses**: IPC overhead, language constraints
- **HTTP callbacks**: Network latency, failure modes
- **Scripting engines**: Limited language support, security concerns

## Decision
We keep **WebAssembly (Wasm)** as a future extension-plane direction. It is not part of the current backend reference path.

### Wasm Components
```
packages/wasm/
├── wit/              # WebAssembly Interface Types definitions
├── host/             # Wasm host runtime (Wasmtime/Wasmer)
├── guest-sdk/        # SDK for writing Wasm plugins
└── components/       # Pre-built Wasm components
    ├── sync-strategy/    # Custom synchronization logic
    ├── protocol-transform/ # Protocol transformation plugins
    └── policy-hook/      # Custom policy evaluation
```

### Architecture
```
Service Logic
  └── Policy Engine
        └── Wasm Host
              └── Wasm Component (tenant-specific plugin)
                    └── WIT Interface (contract)
```

### WIT Interfaces
- Define the contract between host and guest
- Type-safe: Strong typing across language boundaries
- Versioned: Interface evolution is explicit
- Language-agnostic: Guest can be Rust, TypeScript, Go, etc.

### Use Cases
1. **Sync Strategy**: Custom conflict resolution, merge strategies
2. **Protocol Transform**: Parse and transform external protocols
3. **Policy Hook**: Tenant-specific authorization, validation rules
4. **Event Transform**: Custom event enrichment, filtering
5. **Webhook Handler**: Custom webhook logic without core changes

### Security Model
- **Sandboxed**: Wasm runs in isolated memory space
- **Capability-based**: Components only get explicitly granted capabilities
- **Resource limits**: CPU, memory, execution time limits
- **No arbitrary I/O**: Components can't access filesystem or network directly

### Implementation Strategy
- Phase 1: WIT definitions and host runtime scaffold
- Phase 2: Guest SDK for plugin development
- Phase 3: Component implementations for core use cases
- Phase 4: Plugin loading and hot-reloading
- Phase 5: Plugin marketplace and management UI

### Rationale
1. **Security**: Sandboxed execution prevents arbitrary code execution
2. **Performance**: Near-native performance, no IPC overhead
3. **Language flexibility**: Write plugins in any Wasm-targeting language
4. **Versioning**: WIT interfaces enable safe interface evolution
5. **Ecosystem**: Growing Wasm ecosystem (Component Model, WASI)

## Consequences
### What becomes easier
- Tenant customization: Deploy tenant-specific plugins
- Protocol support: Add new protocols without core changes
- Safety: Sandboxed plugins can't crash the host
- Performance: No network calls for plugin execution
- Development: Write plugins in familiar languages

### What becomes more difficult
- Complexity: Wasm tooling is still maturing
- Debugging: Debugging Wasm components requires specialized tools
- Onboarding: Developers need to learn Wasm concepts
- Testing: Need to test plugin loading, execution, sandboxing

### Trade-offs
- **Pros**: Security, performance, language flexibility, versioning
- **Cons**: Complexity, immature tooling, learning curve

### Implementation Status
- ✅ Wasm package structure defined
- ✅ Component use cases identified
- ⏳ WIT definitions not yet implemented
- ⏳ Host runtime not yet implemented
- ⏳ Guest SDK not yet implemented

## References
- `packages/wasm/` - Wasm package structure
- [WebAssembly Component Model](https://github.com/WebAssembly/component-model)
- [WIT (Wasm Interface Type)](https://github.com/WebAssembly/component-model/blob/main/design/mvp/WIT.md)
- [Wasmtime](https://wasmtime.dev/)
- [WASI (WebAssembly System Interface)](https://wasi.dev/)
