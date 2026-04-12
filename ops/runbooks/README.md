# Operations Runbooks

This directory contains runbooks for common operational tasks.

## Available Runbooks

| Runbook | Purpose |
|---------|---------|
| [Backup & Restore](backup-restore.md) | Database backup and restore procedures |
| [Service Deployment](service-deployment.md) | How to deploy services |
| [Incident Response](incident-response.md) | Handling production incidents |
| [Scaling Guide](scaling.md) | Scaling services up and down |
| [Health Checks](health-checks.md) | Verifying system health |

## Quick Reference

### Start Local Infrastructure
```bash
bash infra/local/scripts/bootstrap.sh up
```

### Run Migrations
```bash
bash ops/migrations/runner/migrate.sh up local
```

### Deploy to Kubernetes
```bash
kubectl apply -k infra/k3s/overlays/dev
```

### Check Service Health
```bash
kubectl get pods -n app
kubectl describe pod <pod-name> -n app
kubectl logs <pod-name> -n app
```

### View Logs
```bash
# Local infrastructure
bash infra/local/scripts/bootstrap.sh logs

# Kubernetes
kubectl logs -n app -l app=api
```

## Emergency Contacts

- Infrastructure Team: #infra Slack channel
- On-Call: PagerDuty rotation
- Escalation: See [Incident Response](incident-response.md)
