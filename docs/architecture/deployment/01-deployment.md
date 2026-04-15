# Deployment Diagram

> Shows how the system is deployed across different environments.

## Local Development Topology

```mermaid
graph TB
    subgraph "Developer Machine"
        subgraph "Browser"
            WebApp[Web App\nSvelteKit dev server :5173]
        end
        
        subgraph "Tauri Desktop"
            DesktopApp[Desktop App\nTauri dev]
        end
        
        subgraph "Application Processes"
            WebBFF[Web BFF\nAxum :3000]
            AdminBFF[Admin BFF\nAxum :3001]
        end
        
        subgraph "Workers (optional)"
            Indexer[Indexer Worker]
            Outbox[Outbox Relay]
        end
        
        subgraph "Docker Compose"
            NATS[(NATS\n:4222)]
            Valkey[(Valkey\n:6379)]
            MinIO[(MinIO\nAPI :9000, Console :9001)]
            sqld[(sqld (optional)\nHTTP :8080, gRPC :5001)]
        end
        
        subgraph "Embedded"
            libSQL[(Embedded libSQL\nin-process)]
        end
    end

    WebApp -->|HTTP localhost:3000| WebBFF
    DesktopApp -->|IPC| WebBFF
    WebBFF -->|Internal Rust calls| libSQL
    WebBFF -->|Cache| Valkey
    WebBFF -->|Events| NATS
    
    Indexer -->|Consume| NATS
    Outbox -->|Poll| libSQL
    Outbox -->|Publish| NATS
    
    libSQL -.file.-> LocalFS[(Local Filesystem)]
    Valkey -.file.-> LocalFS
    MinIO -.file.-> LocalFS
```

## Single VPS Topology

```mermaid
graph TB
    subgraph "Single VPS (Docker)"
        subgraph "Reverse Proxy"
            Caddy[Caddy/Traefik\nTLS termination, routing]
        end
        
        subgraph "Application Containers"
            WebBFF[Web BFF\ncontainer]
            AdminBFF[Admin BFF\ncontainer]
            Workers[Workers\nIndexer + Outbox + Projector]
        end
        
        subgraph "Infrastructure Containers"
            NATS[(NATS\nJetStream)]
            Valkey[(Valkey\npersistence)]
            MinIO[(MinIO\nS3-compatible)]
            Zitadel[Zitadel (optional)\nAuthN]
            OpenFGA[OpenFGA (optional)\nAuthZ]
            OpenObserve[OpenObserve\nObservability]
            Vector[Vector\nLog/Metrics collector]
        end
        
        subgraph "Storage"
            AppDB[(libSQL\nmounted volume)]
            NATSData[(NATS data)]
            ValkeyData[(Valkey RDB)]
            MinIOData[(MinIO buckets)]
            ZitadelDB[(Zitadel DB)]
            OpenFGADB[(OpenFGA DB)]
            ObsData[(OpenObserve data)]
        end
    end

    User -->|HTTPS| Caddy
    Caddy -->|web-bff:3000| WebBFF
    Caddy -->|minio:9000| MinIO
    Caddy -->|openobserve:5080| OpenObserve
    
    WebBFF -->|internal| libSQL
    WebBFF -->|NATS| NATS
    WebBFF -->|cache| Valkey
    WebBFF -->|storage| MinIO
    WebBFF -->|auth| Zitadel
    WebBFF -->|authz| OpenFGA
    
    Workers -->|consume| NATS
    Workers -->|poll| libSQL
    
    Vector -->|collect| AppDB
    Vector -->|collect| NATS
    Vector -->|forward| OpenObserve
```

## K3s Cluster Topology

```mermaid
graph TB
    subgraph "External"
        DNS[DNS Provider\nCloudflare/Route53]
        LetEncrypt[Let's Encrypt\nCertificate Authority]
    end

    subgraph "K3s Cluster (3+ nodes)"
        subgraph "System Namespace"
            Cilium[Cilium CNI\neBPF networking]
            CoreDNS[CoreDNS\nService discovery]
            MetricsServer[Metrics Server\nHPA support]
        end
        
        subgraph "Ingress Namespace"
            GatewayAPI[Gateway API\nCilium Gateway]
            CertManager[Cert-Manager\nTLS automation]
        end
        
        subgraph "Infrastructure Namespace"
            NATSStateful[NATS StatefulSet\nJetStream cluster]
            ValkeyStateful[Valkey StatefulSet\nSentinel/Cluster]
            MinIOStateful[MinIO StatefulSet\nDistributed mode]
            OpenObserveStateful[OpenObserve StatefulSet\nObservability]
            ZitadelStateful[Zitadel Deployment\nAuthN]
            OpenFGAStateful[OpenFGA Deployment\nAuthZ]
        end
        
        subgraph "Application Namespace: web"
            WebBFFDeploy[Web BFF Deployment\n2+ replicas]
            WebBFFSvc[Web BFF Service]
        end
        
        subgraph "Application Namespace: admin"
            AdminBFFDeploy[Admin BFF Deployment\n1+ replicas]
            AdminBFFSvc[Admin BFF Service]
        end
        
        subgraph "Workers Namespace"
            IndexerDeploy[Indexer Worker Deployment\nHPA based on stream lag]
            OutboxDeploy[Outbox Relay Deployment\n1 replica (leader elected)]
            ProjectorDeploy[Projector Deployment\nHPA based on event lag]
            SchedulerDeploy[Scheduler Deployment\n1 replica (cron)]
            SyncReconcilerDeploy[Sync Reconciler Deployment\nHPA based on conflicts]
        end
        
        subgraph "GitOps"
            Flux[Flux Controller\nKustomization reconciler]
            SOPS[SOPS\nSecret decryption]
        end
        
        subgraph "Storage"
            Longhorn[Longhorn/LocalPath\nPersistent Volumes]
        end
    end

    DNS -->|DNS| LetEncrypt
    DNS -->|Resolve| GatewayAPI
    
    User -->|HTTPS| GatewayAPI
    GatewayAPI -->|TLS from Cert-Manager| GatewayAPI
    GatewayAPI -->|Route| WebBFFSvc
    GatewayAPI -->|Route| AdminBFFSvc
    
    Cilium -.enforces.-> NetworkPolicy
    
    WebBFFDeploy -->|NATS| NATSStateful
    WebBFFDeploy -->|Cache| ValkeyStateful
    WebBFFDeploy -->|Storage| MinIOStateful
    WebBFFDeploy -->|Auth| ZitadelStateful
    WebBFFDeploy -->|AuthZ| OpenFGAStateful
    
    IndexerDeploy -->|Consume| NATSStateful
    OutboxDeploy -->|Poll| WebBFFDeploy
    ProjectorDeploy -->|Consume| NATSStateful
    
    Flux -->|Reconcile| GatewayAPI
    Flux -->|Decrypt| SOPS
    Flux -->|Deploy| WebBFFDeploy
    Flux -->|Deploy| NATSStateful
```

## Deployment Comparison

| Aspect | Local Dev | Single VPS | K3s Cluster |
|--------|----------|-----------|------------|
| **Orchestration** | docker-compose / k3d | Docker | K3s + Flux |
| **Database** | Embedded libSQL | libSQL file | libSQL/Turso |
| **Message Broker** | NATS (container) | NATS (container) | NATS StatefulSet |
| **Cache** | Valkey (container) | Valkey (container) | Valkey StatefulSet |
| **Object Storage** | MinIO (container) | MinIO (container) | MinIO StatefulSet |
| **AuthN** | MockOAuthProvider | Zitadel (optional) | Zitadel |
| **AuthZ** | In-memory | OpenFGA (optional) | OpenFGA |
| **Observability** | Not started | OpenObserve + Vector | OpenObserve + Vector |
| **TLS** | None | Caddy/Traefik | Cert-Manager |
| **Scaling** | Single instance | Vertical | Horizontal (HPA) |
| **Secrets** | SOPS + age (no .env) | SOPS + age | SOPS + Flux + age |
| **Deployment** | docker-compose up / sops-run | docker-compose + scripts | Flux GitOps |
