# Core

Pure Rust business logic — domain ports and supporting infrastructure.

## Structure

```
core/
├── domain/         # Port trait definitions (LibSQLPort, SurrealDBPort, etc.)
├── workspace-hack/ # cargo-hakari unified dependency crate (build time optimization)
└── state/          # ⚠️ 占位：Shared state machine / cache strategy（待实现）
```

## Design

### Domain Layer

Defines **what** the system needs (traits), not **how** it's done:

```rust
pub trait LibSqlPort: Send + Sync {
    async fn execute(&self, sql: &str, params: Vec<String>) -> Result<(), LibsqlError>;
    async fn query<T: DeserializeOwned>(&self, sql: &str, params: Vec<String>) -> Result<Vec<T>, LibsqlError>;
}
```

### Key Rule

**Ports depend on traits, not concrete implementations.** This means:
- Swap Turso → SurrealDB → Postgres by changing the adapter
- Service code is unchanged
- Full unit test coverage without a real database

> ⚠️ `core/usecases/` 已删除。业务逻辑应实现在 `services/*/application/` 中。
