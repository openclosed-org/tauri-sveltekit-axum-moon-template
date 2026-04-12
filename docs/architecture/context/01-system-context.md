# System Context Diagram

> Shows the system and its external dependencies/interactions.

```mermaid
graph TB
    User((User))
    Admin((Admin))
    OAuthProvider[(OAuth Provider\nGoogle/GitHub/etc.)]
    Web3Protocol[(Web3 Protocols\nNostr/Farcaster/EVM)]

    sub System["Tauri-SvelteKit-Axum Platform"]
        WebApp[Web Application]
        DesktopApp[Desktop Application]
        MobileApp[Mobile Application]
        BrowserExt[Browser Extension]
        
        WebBFF[Web BFF]
        AdminBFF[Admin BFF]
        EdgeGateway[Edge Gateway]
        
        UserService[User Service]
        TenantService[Tenant Service]
        AuthService[Auth Service]
        SettingsService[Settings Service]
        CounterService[Counter Service]
        AdminService[Admin Service]
        IndexingService[Indexing Service]
        
        IndexerWorker[Indexer Worker]
        OutboxRelay[Outbox Relay]
        Projector[Projector]
        Scheduler[Scheduler]
        SyncReconciler[Sync Reconciler]
        
        NATS[(NATS\nMessage Broker)]
        Turso[(Turso/libSQL\nDatabase)]
        Valkey[(Valkey\nCache)]
        MinIO[(MinIO\nObject Storage)]
    end

    User -->|HTTPS| WebApp
    User -->|Native| DesktopApp
    User -->|Native| MobileApp
    User -->|Install| BrowserExt
    
    Admin -->|HTTPS| AdminBFF
    
    WebApp -->|HTTP| WebBFF
    DesktopApp -->|IPC| DesktopApp
    MobileApp -->|HTTP| WebBFF
    BrowserExt -->|HTTP| WebBFF
    
    WebBFF -->|Internal| UserService
    WebBFF -->|Internal| TenantService
    WebBFF -->|Internal| AuthService
    WebBFF -->|Internal| SettingsService
    WebBFF -->|Internal| CounterService
    
    AdminBFF -->|Internal| AdminService
    EdgeGateway -->|Route| WebBFF
    EdgeGateway -->|Route| AdminBFF
    
    UserService -->|Events| NATS
    TenantService -->|Events| NATS
    AuthService -->|Events| NATS
    
    IndexerWorker -->|Consume| NATS
    OutboxRelay -->|Poll| Turso
    OutboxRelay -->|Publish| NATS
    Projector -->|Consume| NATS
    Scheduler -->|Trigger| NATS
    SyncReconciler -->|Reconcile| Turso
    
    UserService -->|Store| Turso
    TenantService -->|Store| Turso
    AuthService -->|Store| Turso
    SettingsService -->|Store| Turso
    
    UserService -->|Cache| Valkey
    AuthService -->|Cache| Valkey
    
    SettingsService -->|Store| MinIO
    
    System <-->|OAuth Flow| OAuthProvider
    System <-->|Sync Data| Web3Protocol
```

## External Dependencies

| System | Purpose | Protocol |
|--------|---------|----------|
| OAuth Provider | User authentication | OAuth2/OIDC |
| Web3 Protocols | Data synchronization | Nostr/Farcaster/AT Protocol/EVM |

## External Actors

| Actor | Interaction |
|-------|-------------|
| User | Uses web, desktop, mobile, browser extension |
| Admin | Manages platform via admin BFF |
| OAuth Provider | Handles external authentication |
| Web3 Protocol | Provides external data sources |

## Internal Systems

| System | Description |
|--------|-------------|
| Web BFF | Aggregates services for web clients |
| Admin BFF | Admin-facing API aggregation |
| Edge Gateway | Edge routing, rate limiting, auth |
| Services | Business logic libraries |
| Workers | Async processing units |
| NATS | Message broker with JetStream |
| Turso | libSQL database |
| Valkey | Redis-compatible cache |
| MinIO | S3-compatible object storage |
