# Sync Flow Diagram

> Shows the offline-first synchronization architecture.

```mermaid
graph TB
    subgraph "Client (Web/Desktop/Mobile)"
        LocalUI[Local UI\nOptimistic Updates]
        LocalDB[(Local DB\nIndexedDB/Tauri Store)]
        SyncEngine[Sync Engine\nCRDT/Merge Logic]
        ConflictResolver[Conflict Resolver\nPolicy-driven]
        
        LocalUI --> LocalDB
        LocalDB --> SyncEngine
        SyncEngine --> ConflictResolver
        ConflictResolver --> LocalDB
    end
    
    subgraph "Network Layer"
        SyncAPI[Sync API\nPOST /sync]
        Compression[Delta Compression]
        Retry[Retry Logic\nExponential Backoff]
        
        SyncEngine --> SyncAPI
        SyncAPI --> Compression
        Compression --> Retry
    end
    
    subgraph "Server (Web BFF)"
        SyncHandler[Sync Handler\nDelta Processing]
        DedupEngine[Dedup Engine\nIdempotency]
        MergeEngine[Merge Engine\nServer-side Merge]
        ServerConflict[Server Conflict Detector]
        
        Retry --> SyncHandler
        SyncHandler --> DedupEngine
        DedupEngine --> MergeEngine
        MergeEngine --> ServerConflict
    end
    
    subgraph "Server Database"
        ServerDB[(Turso/libSQL\nServer DB)]
        OutboxTable[Outbox Table\nChange Feed]
        
        MergeEngine --> ServerDB
        ServerDB --> OutboxTable
    end
    
    subgraph "Async Processing"
        OutboxRelay[Outbox Relay\nPoll & Publish]
        NATS[NATS JetStream\nEvent Stream]
        Projector[Projector\nRead Model Update]
        SyncReconciler[Sync Reconciler\nDrift Detection]
        
        OutboxTable --> OutboxRelay
        OutboxRelay --> NATS
        NATS --> Projector
        NATS --> SyncReconciler
        
        Projector --> ServerDB
        SyncReconciler --> ServerDB
    end
    
    ServerDB -.response.-> MergeEngine
    MergeEngine -.server changes.-> SyncEngine
    SyncReconciler -.conflict resolution.-> ConflictResolver
```

## Sync States

```mermaid
stateDiagram-v2
    [*] --> Online
    Online --> Offline: Network loss
    Offline --> Online: Network restored
    Online --> Syncing: User action or timer
    Syncing --> Online: Sync complete
    Syncing --> Conflict: Conflicts detected
    Conflict --> Syncing: Conflicts resolved
    Conflict --> Online: Conflicts resolved (no changes needed)
    Offline --> LocalOnly: Local changes made
    LocalOnly --> Offline: More local changes
    LocalOnly --> Syncing: Network restored + sync triggered
    Syncing --> Offline: Network lost during sync
    Syncing --> Conflict: Server rejects with conflict
```

## Sync Strategies

| Strategy | Use Case | Conflict Resolution |
|----------|----------|-------------------|
| **Last-Write-Wins** | Simple fields, settings | Timestamp comparison |
| **Field-Level Merge** | Documents, forms | Per-field merge |
| **CRDT** | Counters, sets, maps | Mathematical convergence |
| **Custom (Wasm)** | Tenant-specific rules | Plugin-based resolution |
| **Manual** | Critical data | User selection required |
