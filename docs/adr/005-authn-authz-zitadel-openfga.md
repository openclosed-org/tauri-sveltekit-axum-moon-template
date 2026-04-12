# ADR-005: AuthN/AuthZ with Zitadel + OpenFGA

## Status
- [x] Proposed
- [x] Accepted
- [ ] Deprecated
- [ ] Superseded

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
We selected **Zitadel for Authentication** and **OpenFGA for Authorization**:

### Authentication (Zitadel)
- OIDC/OAuth2 compliant: Standard protocols
- Multi-tenancy: Native tenant support
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
- Auditable: Decision logging and tracing

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
- **Production**: Replace with Zitadel + real OAuth providers
- **Authorization**: OpenFGA for both dev (embedded) and prod (clustered)

### Rationale
1. **Security**: Auth is critical; battle-tested solutions are safer
2. **Flexibility**: Zitadel + OpenFGA covers both authN and authZ
3. **Self-hosted**: No vendor lock-in, full data control
4. **Modern**: Both projects are actively developed
5. **Standard**: OIDC/OAuth2 ensures compatibility

## Consequences
### What becomes easier
- Multi-tenancy: Native support from Zitadel
- Fine-grained authZ: OpenFGA relationship tuples
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
- ⏳ Production OAuth provider (Zitadel integration) deferred
- ⏳ OpenFGA adapter deferred

## References
- `services/auth-service/src/infrastructure/` - Auth infrastructure adapters
- `packages/authn/` - Authentication package
- `packages/authz/` - Authorization package
- [Zitadel Documentation](https://zitadel.com/docs/)
- [OpenFGA Documentation](https://openfga.dev/)
