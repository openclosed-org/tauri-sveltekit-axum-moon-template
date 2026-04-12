# Topology Diagram

> Shows how the same platform can be deployed in different topological configurations.

## Topology Overview

The platform is topology-agnostic. The same services, deployables, and resources can be assembled into different topologies without changing business logic. Topology is defined in `platform/model/topologies/*.yaml`.

```mermaid
graph LR
    PlatformModel[Platform Model\nservices/deployables/resources] --> TopologySelector{Topology\nSelector}
    
    TopologySelector -->|local-dev| LocalTopo[Local Dev Topology]
    TopologySelector -->|single-vps| VPSTopology[Single VPS Topology]
    TopologySelector -->|k3s-staging| StagingTopo[Staging Topology]
    TopologySelector -->|k3s-microservices| MicroTopo[Microservices Topology]
    
    LocalTopo --> LocalGen[Generate docker-compose]
    VPSTopology --> VPSGen[Generate docker-compose + configs]
    StagingTopo --> StagingGen[Generate K8s manifests]
    MicroTopo --> MicroGen[Generate K8s manifests + Flux]
```

## Topology: Local Development

```mermaid
graph TB
    subgraph "Host Machine"
        Browser[Browser\nlocalhost:5173]
        TauriApp[Tauri Desktop App]
    end
    
    subgraph "Docker Compose: core"
        NATS[NATS:4222]
        Valkey[Valkey:6379]
        MinIO[MinIO:9000/9001]
    end
    
    subgraph "In-Process"
        WebBFF[Web BFF\n:3000]
        AdminBFF[Admin BFF\n:3001]
        Services[(All Services\nas libraries)]
        Workers[(Workers\nin-process)]
        libSQL[(Embedded libSQL\nfile-based)]
    end

    Browser --> WebBFF
    TauriApp --> WebBFF
    WebBFF --> Services
    Services --> libSQL
    Services --> NATS
    Services --> Valkey
    Workers --> NATS
    Workers --> libSQL
```

### Characteristics
| Aspect | Detail |
|--------|--------|
| Processes | 2 (web-bff, admin-bff) + docker services |
| Database | Embedded (no separate process) |
| Workers | Optional, in-process or separate |
| Auth | MockOAuthProvider |
| AuthZ | In-memory decision engine |
| Network | localhost only |
| Persistence | File-based (libSQL, Valkey RDB, MinIO volumes) |

## Topology: Single VPS

```mermaid
graph TB
    User((User))
    
    subgraph "VPS (Docker)"
        subgraph "Edge"
            Caddy[Caddy\n:80/:443]
        end
        
        subgraph "Application"
            WebBFF[Web BFF\n:3000]
            AdminBFF[Admin BFF\n:3001]
            Workers[Workers\n:4000-4005]
        end
        
        subgraph "Infrastructure"
            NATS[NATS\n:4222]
            Valkey[Valkey\n:6379]
            MinIO[MinIO\n:9000/:9001]
            Zitadel[Zitadel\n:8080]
            OpenFGA[OpenFGA\n:8081]
            OpenObserve[OpenObserve\n:5080]
            Vector[Vector\nlog collector]
        end
        
        subgraph "Storage"
            Volumes[(Docker Volumes\npersistent data)]
        end
    end

    User -->|HTTPS| Caddy
    Caddy --> WebBFF
    Caddy --> AdminBFF
    Caddy --> OpenObserve
    
    WebBFF --> NATS
    WebBFF --> Valkey
    WebBFF --> MinIO
    WebBFF --> Zitadel
    WebBFF --> OpenFGA
    
    Workers --> NATS
    Workers --> Vector
    
    NATS --> Volumes
    Valkey --> Volumes
    MinIO --> Volumes
```

### Characteristics
| Aspect | Detail |
|--------|--------|
| Processes | 8-10 containers |
| Database | libSQL file or Turso server |
| Workers | Separate containers |
| Auth | Zitadel (self-hosted) |
| AuthZ | OpenFGA (self-hosted) |
| Network | Docker network, Caddy reverse proxy |
| Scaling | Vertical only (increase VPS resources) |

## Topology: K3s Staging

```mermaid
graph TB
    User((User))
    DNS[DNS]
    
    subgraph "K3s Cluster (1-3 nodes)"
        subgraph "ingress"
            Gateway[Cilium Gateway\n+ Cert-Manager]
        end
        
        subgraph "app namespace"
            WebBFF[Web BFF\n2 replicas]
            AdminBFF[Admin BFF\n1 replica]
        end
        
        subgraph "workers namespace"
            Indexer[Indexer]
            Outbox[Outbox Relay]
            Projector[Projector]
        end
        
        subgraph "infra namespace"
            NATS[NATS\n3 replicas]
            Valkey[Valkey\n3 replicas]
            MinIO[MinIO\n4 replicas]
            Zitadel[Zitadel\n1 replica]
            OpenFGA[OpenFGA\n1 replica]
        end
        
        subgraph "GitOps"
            Flux[Flux\nSOPS decryption]
        end
        
        subgraph "Storage"
            PVCs[(PersistentVolumeClaims\nLonghorn/LocalPath)]
        end
    end

    DNS --> Gateway
    User -->|HTTPS| Gateway
    Gateway --> WebBFF
    Gateway --> AdminBFF
    
    WebBFF --> NATS
    WebBFF --> Valkey
    WebBFF --> Zitadel
    WebBFF --> OpenFGA
    
    Indexer --> NATS
    Outbox --> NATS
    Projector --> NATS
    
    Flux --> Gateway
    Flux --> WebBFF
    Flux --> NATS
    
    NATS --> PVCs
    Valkey --> PVCs
    MinIO --> PVCs
```

### Characteristics
| Aspect | Detail |
|--------|--------|
| Nodes | 1-3 (staging) |
| Processes | Kubernetes Pods |
| Replicas | Minimal (1-2 per service) |
| Workers | Separate namespace |
| Auth | Zitadel |
| AuthZ | OpenFGA |
| Network | Cilium CNI, Gateway API |
| Scaling | Manual or basic HPA |
| GitOps | Flux with SOPS |

## Topology: K3s Microservices (Production)

```mermaid
graph TB
    User((User))
    DNS[DNS + CDN]
    
    subgraph "K3s Cluster (5+ nodes)"
        subgraph "ingress"
            Gateway[Cilium Gateway\n+ Cert-Manager\n+ External DNS]
        end
        
        subgraph "web namespace"
            WebBFF1[Web BFF\n3+ replicas\nHPA: CPU/RAM]
            WebBFF2[Web BFF\n3+ replicas]
        end
        
        subgraph "admin namespace"
            AdminBFF1[Admin BFF\n2+ replicas]
        end
        
        subgraph "workers namespace"
            Indexer1[Indexer\nHPA: stream lag]
            Indexer2[Indexer\nHPA: stream lag]
            Outbox[Outbox Relay\nleader elected]
            Projector[Projector\nHPA: event lag]
            Scheduler[Scheduler\n1 replica]
            SyncReconciler[Sync Reconciler\nHPA: conflicts]
        end
        
        subgraph "infra namespace"
            NATS1[NATS\n3 replicas\nJetStream cluster]
            Valkey1[Valkey\n3 replicas\ncluster]
            MinIO1[MinIO\n4+ replicas\ndistributed]
            Zitadel1[Zitadel\n2+ replicas\nHA]
            OpenFGA1[OpenFGA\n2+ replicas\nHA]
            OpenObserve1[OpenObserve\n2+ replicas]
            Vector1[Vector\nDaemonSet]
        end
        
        subgraph "GitOps namespace"
            Flux[Flux\nmulti-tenancy\nSOPS + Kustomize]
        end
        
        subgraph "observability"
            Hubble[Hubble\nnetwork observability]
            Grafana[Grafana\ndashboards]
        end
        
        subgraph "Storage"
            PVCs[(PersistentVolumeClaims\nLonghorn/Ceph)]
            Backup[(Backup\nVelero/CronJob)]
        end
    end

    DNS --> Gateway
    User -->|HTTPS| Gateway
    Gateway --> WebBFF1
    Gateway --> WebBFF2
    Gateway --> AdminBFF1
    
    WebBFF1 --> NATS1
    WebBFF2 --> NATS1
    AdminBFF1 --> NATS1
    
    Indexer1 --> NATS1
    Indexer2 --> NATS1
    Outbox --> NATS1
    Projector --> NATS1
    
    Vector1 --> OpenObserve1
    
    Flux -->|reconcile| Gateway
    Flux -->|reconcile| WebBFF1
    Flux -->|reconcile| NATS1
    
    NATS1 --> PVCs
    Valkey1 --> PVCs
    MinIO1 --> PVCs
    PVCs --> Backup
```

### Characteristics
| Aspect | Detail |
|--------|--------|
| Nodes | 5+ (production) |
| Processes | Kubernetes Pods with HPA |
| Replicas | 2-3+ per service (HA) |
| Workers | Independent scaling per worker |
| Auth | Zitadel HA |
| AuthZ | OpenFGA HA |
| Network | Cilium CNI, Gateway API, Hubble |
| Scaling | HPA (CPU, memory, custom metrics) |
| GitOps | Flux with Kustomize + SOPS |
| Backup | Velero or CronJob-based |
| Monitoring | OpenObserve + Hubble + Grafana |

## Topology Switching

The same platform model supports all topologies. Switching topology only requires:

1. Edit `platform/model/topologies/<name>.yaml`
2. Run `just render-k8s env=dev` or `just render-local`
3. Deploy generated manifests

**No business logic changes required.**
