# ADR-010: Generic OIDC With Rauthy + OpenFGA Local Auth Lane

## Status
- [ ] Proposed
- [x] Accepted
- [ ] Deprecated
- [ ] Superseded

## Context

The repository needs an optional local auth lane for backend resource-server testing without making a specific IdP part of `web-bff` runtime semantics.

The current backend architecture keeps `services/**` as business libraries and `servers/**` as protocol entrypoints. Authentication provider details belong in infra/bootstrap/docs, while server runtime code should depend on generic OIDC concepts: issuer, audience, discovery, JWKS, introspection, and verified identity.

Earlier local auth work used Zitadel-specific bootstrap state and `APP_ZITADEL_*` env names. That made the local provider look like a runtime contract and made Rauthy/Keycloak/Kanidm/Auth0 replacement harder than necessary.

## Decision

Use Generic OIDC as the `web-bff` resource-server contract and use Rauthy as the local reference IdP for the optional auth lane.

Runtime-facing env must stay provider-neutral:

1. `APP_OIDC_ISSUER`
2. `APP_OIDC_AUDIENCE`
3. `APP_OIDC_INTROSPECTION_URL`
4. `APP_OIDC_INTROSPECTION_CLIENT_ID`
5. `APP_OIDC_INTROSPECTION_CLIENT_SECRET`
6. `APP_AUTHZ_PROVIDER`
7. `APP_AUTHZ_ENDPOINT`
8. `APP_AUTHZ_STORE_ID`
9. `APP_AUTHZ_MODEL_ID`

Provider-specific state files may use provider names, such as `infra/local/state/rauthy.*` and `infra/local/state/openfga.*`, because those files are local bootstrap state, not application runtime API.

## Consequences

`web-bff` remains provider-neutral and imports the shared `authn-oidc-verifier` package for token verification.

`infra/docker/compose/auth.yaml` owns the local auth provider selection and pins Rauthy/OpenFGA images.

`repo-tools infra auth bootstrap` writes generic `APP_OIDC_*` / `APP_AUTHZ_*` env and no longer writes provider-specific app runtime env such as `APP_ZITADEL_*` or `APP_OPENFGA_*`.

Zitadel is historical context for this repository. It can appear in migration notes or archived rationale, but it is not the current default local auth stack.

## Verification

Expected guardrails after auth-lane changes:

1. `rtk cargo check -p repo-tools -p web-bff`
2. `rtk cargo test -p repo-tools -p web-bff`
3. grep current entry docs and runtime sources for `APP_ZITADEL_*`
4. optional smoke: `cargo run -p repo-tools -- infra auth bootstrap`

## References

1. `packages/authn/oidc-verifier`
2. `packages/authz`
3. `infra/docker/compose/auth.yaml`
4. `tools/repo-tools/src/commands/infra.rs`
