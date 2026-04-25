# ADR-005: AuthN/AuthZ with Zitadel + OpenFGA

## Status
- [x] Proposed
- [ ] Accepted
- [ ] Deprecated
- [ ] Superseded

> **Implementation Status**: Local Zitadel resource-server validation, OpenFGA adapter, and Podman bootstrap scripts now exist for optional local/backend auth flows.
> They are not the default backend admission path. The canonical backend reference remains the counter chain, and auth stays an optional lane.

## Context
The system needs robust authentication and authorization capabilities that support:
- Multi-tenant isolation
- OAuth/OIDC flows (Google, GitHub, etc.)
- Role-based access control (RBAC)
- Attribute-based access control (ABAC)
- Session management
- Token validation and refresh
- Fine-grained authorization decisions

Building auth from scratch is risky and time-consuming. Options considered:
1. **Custom auth**: Full control but high risk and maintenance
2. **Keycloak**: Powerful but heavy and complex
3. **Auth0**: Easy but expensive and vendor lock-in
4. **Zitadel + OpenFGA**: Modern, open-source, flexible
5. **Supabase Auth**: Easy but limited customization

## Decision
We keep **Zitadel + OpenFGA** as the preferred future auth platform direction, but it is not yet part of the default backend reference path.

### Authentication (Zitadel)
- OIDC/OAuth2 compliant: Standard protocols
- Multi-tenancy: a target capability, not a current repository guarantee
- User management: Registration, password reset, MFA
- Session management: Secure session handling
- Self-hosted: Full control over user data
- Modern UI: Login screens, consent flows
- API-first: Programmatic user/tenant management

### Authorization (OpenFGA)
- Fine-grained authorization: Beyond simple RBAC
- Relationship-based: Users, groups, resources, roles
- Zanzibar-inspired: Proven architecture (used by Google, Auth0)
- Fast: Sub-millisecond authorization decisions
- Self-hosted: No external API calls
- DSL: Clear policy language
- Auditable: a target capability, not a current repository guarantee

### Implementation Strategy
```
packages/authn/
├── oidc/           # OIDC flow handling
├── pkce/           # PKCE for SPA/mobile
├── session/        # Session management
└── token/          # JWT token handling

packages/authz/
├── model/          # Authorization model definition
├── ports/          # Abstract authorization interface
├── caching/        # Decision caching
├── decision/       # Decision engine
└── adapters/openfga/  # OpenFGA adapter

services/auth-service/
├── infrastructure/
│   ├── jwt_token_repository.rs      # JWT token storage
│   ├── libsql_session_repository.rs # Session storage
│   └── mock_oauth_provider.rs       # Mock OAuth for dev
└── ...
```

### Development vs Production
- **Development**: MockOAuthProvider for local testing
- **Backend-first local development**: `web-bff` may use `APP_AUTH_MODE=dev_headers` to debug handler/service flows without OAuth bootstrap
- **Production**: Replace with Zitadel + real OAuth providers when auth becomes part of the chosen deployable path
- **Authorization**: OpenFGA is available as an optional integration path, not a default prerequisite for the primary backend lane

### Rationale
1. **Security**: Auth is critical; battle-tested solutions are safer
2. **Flexibility**: Zitadel + OpenFGA covers both authN and authZ
3. **Self-hosted**: No vendor lock-in, full data control
4. **Modern**: Both projects are actively developed
5. **Standard**: OIDC/OAuth2 ensures compatibility

## Consequences
### What becomes easier
- Future multi-tenancy integration once auth platform work is actually scheduled
- Fine-grained authZ once OpenFGA integration is actually implemented
- Local dev: MockOAuthProvider for testing
- Security: OIDC standard, proven implementations
- Auditing: OpenFGA decision logs

### What becomes more difficult
- Infrastructure: Two additional services to deploy
- Complexity: Auth flows are inherently complex
- Migration: Moving auth state between systems is hard
- Integration: OAuth provider setup requires external accounts

### Trade-offs
- **Pros**: Security, flexibility, self-hosted, standards-compliant
- **Cons**: Infrastructure overhead, complexity, integration effort

### Implementation Status
- ✅ JWT token repository implemented
- ✅ LibSQL session repository implemented
- ✅ MockOAuthProvider for development
- ✅ `web-bff` supports Zitadel OIDC/JWKS and introspection-based resource-server validation
- ✅ `packages/authz` includes an OpenFGA adapter
- ✅ Local Podman bootstrap exists for Zitadel + OpenFGA (`infra/docker/compose/auth.yaml`, `infra/local/scripts/bootstrap-auth.sh`)
- ⏳ Interactive end-user frontend OIDC login is still not the default repository path

## References
- `services/auth-service/src/infrastructure/` - Auth infrastructure adapters
- `packages/authn/` - Authentication package
- `packages/authz/` - Authorization package
- [Zitadel Documentation](https://zitadel.com/docs/)
- [OpenFGA Documentation](https://openfga.dev/)
