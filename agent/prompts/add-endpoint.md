# Agent Prompt — Add Endpoint

## Steps

1. **Identify target BFF**: web-bff, mobile-bff, or admin-bff
2. **Define contract**: Add request/response types to `packages/contracts/http/`
3. **Create handler**: Use `agent/templates/bff-endpoint/src/handler.rs.gitkeep` as template
4. **Create adapter**: Use `agent/templates/bff-endpoint/src/adapter.rs.gitkeep` as template
5. **Wire route**: Add to BFF's router in `apps/bff/<bff-name>/src/main.rs`
6. **Update OpenAPI**: Add utoipa annotations, regenerate with `just gen-openapi`
7. **Generate frontend SDK**: Run `just gen-frontend-sdk`
8. **Verify**:
   - `cargo build -p <bff-name>` must succeed
   - `just gen-openapi && git diff --exit-code` must pass
   - Frontend compiles with new types

## Checklist

- [ ] Handler only does adaptation (auth extraction, validation, response mapping)
- [ ] Handler delegates to service trait, no domain logic
- [ ] Adapter implements service port trait
- [ ] OpenAPI annotations complete and accurate
- [ ] Error responses use shared error types
- [ ] Auth/tenant middleware applied if needed
