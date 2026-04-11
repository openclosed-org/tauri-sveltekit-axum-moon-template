# infra/terraform — Infrastructure as Code (Phase 2+)

## Planned Structure

```
terraform/
├── modules/
│   ├── vpc/            # Network topology
│   ├── ec2/            # Compute instances (k3s nodes)
│   ├── turso/          # Turso/LibSQL database cluster
│   ├── surrealdb/      # SurrealDB cluster (optional, for prod scale)
│   └── alb/            # Application load balancer
└── environments/
    ├── dev/            # Development overrides
    └── prod/           # Production configuration
```

## Status

- [ ] Phase 2: Implement Terraform modules
- [ ] Phase 2: Set up remote state backend (S3 + DynamoDB)
- [ ] Phase 2: Configure environment overrides
- [ ] Phase 3: Multi-region deployment

## Current Alternative

For Phase 1, use k3s on a single VPS with Podman:

```bash
just deploy bootstrap-k3s
just deploy prod ENV=dev
```

## Stack Notes

- **No PostgreSQL RDS** — We use Turso (libSQL) and SurrealDB
- **No Elasticache/Redis** — We use Moka (in-process cache)
- **No NATS cluster** — Phase 1 uses in-process event bus; Phase 2 may add NATS if needed
