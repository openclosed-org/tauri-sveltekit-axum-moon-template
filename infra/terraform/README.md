# infra/terraform — Planned Infrastructure As Code

This directory is a planning placeholder. It is not the current default deployment path and does not prove cloud infrastructure readiness.

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

- Terraform modules are not implemented.
- Remote state, environment overrides, and multi-region deployment are not wired.
- Current delivery evidence should come from `infra/k3s/**`, `infra/gitops/flux/**`, SOPS templates, validators, and executed gates.

## Current Alternative

For current cluster-shape work, start from K3s and GitOps documentation instead of this Terraform placeholder:

```bash
just validate-topology
```

## Stack Notes

- The minimal backend-core path should not require managed PostgreSQL, Redis, or NATS.
- Local and cluster profiles can still include libSQL/Turso-compatible infrastructure, optional SurrealDB, Valkey, NATS, and MinIO.
- Do not use this README to infer production resource choices; verify current topology and deployable declarations instead.
