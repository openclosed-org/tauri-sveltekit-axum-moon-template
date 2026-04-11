# BFF endpoint template

> Use this template when creating new BFF endpoints.

## Steps
1. Copy this template to `servers/{{endpoint-name}}/`
2. Replace `{{bff-name}}` with the target client (web, mobile, admin)
3. Replace `{{target}}` with the client description
4. Implement `handler.rs` with view model logic
5. Implement `adapter.rs` with HTTP mapping
6. Add routes to `servers/api/src/routes/mod.rs`
7. Update `Cargo.toml` with service dependencies
