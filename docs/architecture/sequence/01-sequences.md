# Sequence Diagrams

> Shows key interaction flows between system components.

## 1. OAuth Login Flow

```mermaid
sequenceDiagram
    participant User
    participant WebApp as Web Application
    participant WebBFF as Web BFF
    participant AuthService as Auth Service
    participant OAuth as OAuth Provider
    participant Turso as Turso/libSQL
    participant Valkey as Valkey Cache
    participant NATS as NATS

    User->>WebApp: Click "Login with Google"
    WebApp->>WebBFF: GET /auth/oauth/authorize?provider=google
    WebBFF->>AuthService: GenerateAuthorizationURL(provider)
    AuthService->>AuthService: Generate PKCE code_verifier/challenge
    AuthService-->>WebBFF: Authorization URL + state
    WebBFF-->>WebApp: 302 Redirect to OAuth provider
    WebApp-->>User: Redirect to Google
    
    User->>OAuth: Authenticate with Google
    OAuth-->>User: 302 Redirect to callback URL
    User->>WebBFF: GET /auth/oauth/callback?code=xxx&state=yyy
    WebBFF->>WebBFF: Validate state parameter
    WebBFF->>AuthService: ExchangeCodeForTokens(code, code_verifier)
    AuthService->>OAuth: POST /token (code + PKCE challenge)
    OAuth-->>AuthService: id_token, access_token, refresh_token
    AuthService->>AuthService: Validate id_token
    AuthService->>AuthService: Extract user info from token
    
    AuthService->>Turso: Upsert user (if new)
    Turso-->>AuthService: User record
    
    AuthService->>AuthService: Generate JWT access token (15min)
    AuthService->>AuthService: Generate refresh token (7d)
    AuthService->>Turso: Store session (session_id, user_id, expires_at)
    Turso-->>AuthService: Session stored
    
    AuthService->>AuthService: Create session object
    AuthService-->>WebBFF: Session + tokens
    
    WebBFF->>Valkey: Cache user session
    WebBFF->>NATS: Publish UserLoggedIn event
    WebBFF-->>WebApp: Set-Session cookie + redirect
    WebApp-->>User: Redirect to dashboard
```

## 2. Tenant Onboarding Flow

```mermaid
sequenceDiagram
    participant User
    participant WebApp as Web Application
    participant WebBFF as Web BFF
    participant TenantService as Tenant Service
    participant UserService as User Service
    participant Turso as Turso/libSQL
    participant NATS as NATS
    participant OutboxRelay as Outbox Relay
    participant Projector as Projector Worker

    User->>WebApp: Fill "Create Tenant" form
    WebApp->>WebBFF: POST /tenants {name, slug, plan}
    WebBFF->>WebBFF: Validate request + auth context
    WebBFF->>TenantService: CreateTenant(name, slug, plan, user_id)
    
    TenantService->>TenantService: Validate slug uniqueness
    TenantService->>TenantService: Generate tenant_id (ULID)
    TenantService->>TenantService: Create tenant entity
    TenantService->>TenantService: Create tenant member (owner)
    TenantService->>Turso: Save tenant + member (transaction)
    Turso-->>TenantService: Tenant created
    
    TenantService->>Turso: Write TenantCreated to outbox table
    Turso-->>TenantService: Outbox entry persisted
    
    TenantService-->>WebBFF: Tenant created + tenant_id
    WebBFF-->>WebApp: 201 Created + tenant context
    WebApp-->>User: Show success + redirect to tenant dashboard
    
    Note over OutboxRelay,Turso: Async (Outbox Relay polls)
    OutboxRelay->>Turso: Poll outbox table for unpublished events
    Turso-->>OutboxRelay: TenantCreated event
    
    OutboxRelay->>OutboxRelay: Check idempotency (already published?)
    OutboxRelay->>NATS: Publish TenantCreated to tenants.* subject
    NATS-->>OutboxRelay: Acknowledged
    OutboxRelay->>Turso: Mark event as published
    
    Note over Projector,NATS: Async (Projector consumes)
    Projector->>NATS: Subscribe to tenants.*
    NATS-->>Projector: TenantCreated event
    Projector->>Projector: Build tenant read model
    Projector->>Turso: Update tenant projection
```

## 3. API Request Flow (with Auth + Tenant)

```mermaid
sequenceDiagram
    participant Client as Web/Desktop Client
    participant Gateway as Edge Gateway
    participant WebBFF as Web BFF
    participant AuthMiddleware as Auth Middleware
    participant TenantMiddleware as Tenant Middleware
    participant TelemetryMiddleware as Telemetry Middleware
    participant UserService as User Service
    participant Turso as Turso/libSQL
    participant Valkey as Valkey Cache

    Client->>Gateway: GET /api/users/me (with JWT cookie)
    Gateway->>Gateway: Rate limiting check
    Gateway->>Gateway: TLS termination
    Gateway->>WebBFF: Forward request + headers
    
    WebBFF->>TelemetryMiddleware: Start trace span
    TelemetryMiddleware->>TelemetryMiddleware: Extract trace context
    
    WebBFF->>AuthMiddleware: Validate JWT token
    AuthMiddleware->>AuthMiddleware: Parse token claims
    AuthMiddleware->>AuthMiddleware: Verify signature
    AuthMiddleware->>AuthMiddleware: Check expiration
    AuthMiddleware->>Valkey: Check if session still valid
    Valkey-->>AuthMiddleware: Session valid
    
    AuthMiddleware->>AuthMiddleware: Extract user_id, tenant_id, roles
    AuthMiddleware->>TenantMiddleware: Resolve tenant context
    TenantMiddleware->>TenantMiddleware: Verify user belongs to tenant
    TenantMiddleware->>TenantMiddleware: Set tenant_id in request context
    
    WebBFF->>UserService: GetUser(user_id)
    UserService->>Turso: SELECT * FROM users WHERE id = ?
    Turso-->>UserService: User record
    UserService->>UserService: Build user entity
    UserService-->>WebBFF: User object
    
    WebBFF->>TelemetryMiddleware: Add metrics + end span
    WebBFF-->>Gateway: 200 OK + user JSON
    Gateway-->>Client: 200 OK + response body
```

## 4. Event Processing Flow (Indexer Worker)

```mermaid
sequenceDiagram
    participant Service as Service (e.g., User Service)
    participant OutboxTable as Turso Outbox Table
    participant OutboxRelay as Outbox Relay Worker
    participant NATS as NATS JetStream
    participant Indexer as Indexer Worker
    participant Transform as Transform Pipeline
    participant Sink as Sink (Search Index)
    participant Checkpoint as Checkpoint Store

    Service->>OutboxTable: INSERT event + payload (within tx)
    Service-->>Client: Operation confirmed
    
    loop Every 100ms
        OutboxRelay->>OutboxTable: SELECT unpublished events ORDER BY id LIMIT 100
        OutboxTable-->>OutboxRelay: Unpublished events
    end
    
    OutboxRelay->>OutboxRelay: Dedupe (check inbox table)
    OutboxRelay->>NATS: Publish event to stream
    NATS-->>OutboxRelay: Published
    OutboxRelay->>OutboxTable: UPDATE set published_at = NOW()
    
    Indexer->>NATS: Subscribe to events.* stream
    NATS-->>Indexer: New event
    
    Indexer->>Checkpoint: Get last processed sequence
    Checkpoint-->>Indexer: Sequence N
    
    Indexer->>Indexer: Skip if sequence <= N
    Indexer->>Transform: Process event
    Transform->>Transform: Parse event type
    Transform->>Transform: Enrich with metadata
    Transform->>Transform: Filter/transform payload
    Transform-->>Indexer: Transformed document
    
    Indexer->>Sink: Upsert document
    Sink-->>Indexer: Document indexed
    
    Indexer->>Checkpoint: Save sequence N+1
    Checkpoint-->>Indexer: Checkpoint saved
```

## 5. Sync Reconciliation Flow

```mermaid
sequenceDiagram
    participant LocalDB as Local Database (client)
    participant SyncClient as Sync Client (app)
    participant WebBFF as Web BFF
    participant SyncReconciler as Sync Reconciler Worker
    participant Turso as Turso (server)
    participant ConflictResolver as Conflict Resolver

    SyncClient->>LocalDB: Get local changes since last sync
    LocalDB-->>SyncClient: Local change set
    
    SyncClient->>WebBFF: POST /sync {last_sync_at, changes}
    WebBFF->>Turso: Get server changes since last_sync_at
    Turso-->>WebBFF: Server change set
    
    WebBFF->>SyncReconciler: Reconcile(local_changes, server_changes)
    
    SyncReconciler->>SyncReconciler: Detect conflicts (same record modified both sides)
    
    alt No conflicts
        SyncReconciler->>Turso: Apply local changes
        SyncReconciler->>SyncClient: Return server changes
    else Conflicts detected
        SyncReconciler->>ConflictResolver: Resolve conflicts
        
        alt Last-write-wins policy
            ConflictResolver->>ConflictResolver: Compare updated_at timestamps
            ConflictResolver-->>SyncReconciler: Winner per timestamp
        else Custom policy (Wasm)
            ConflictResolver->>ConflictResolver: Load tenant-specific Wasm plugin
            ConflictResolver->>ConflictResolver: Execute plugin with conflict data
            ConflictResolver-->>SyncReconciler: Plugin resolution result
        end
        
        SyncReconciler->>Turso: Apply resolved changes
        SyncReconciler->>SyncClient: Return resolved changes
    end
    
    WebBFF-->>SyncClient: Sync response {server_changes, conflicts_resolved}
    SyncClient->>LocalDB: Apply server changes + resolve conflicts
    SyncClient->>LocalDB: Update last_sync_at
```
