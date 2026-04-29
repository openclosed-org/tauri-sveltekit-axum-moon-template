# Backup & Restore Runbook

This runbook covers backup and restore procedures for all data stores.

## Overview

| Data Store | Backup Method | Frequency | Retention |
|-----------|--------------|-----------|-----------|
| libSQL/Turso | File-level backup | Daily | 7 days |
| NATS JetStream | Built-in persistence | Continuous | Based on disk space |
| Valkey | RDB snapshots | Hourly | 24 hours |
| MinIO | Built-in replication | Continuous | Unlimited |

## Backup Procedures

### libSQL/Turso Database

#### Local Development

```bash
# Stop the application to ensure clean state
cargo run -p repo-tools -- infra local down

# Backup database file
BACKUP_DIR="./backups/$(date +%Y%m%d_%H%M%S)"
mkdir -p "$BACKUP_DIR"
cp .data/*.db "$BACKUP_DIR/" 2>/dev/null || echo "No database files found"

# Restart services
cargo run -p repo-tools -- infra local up

log_success "Database backed up to $BACKUP_DIR"
```

#### Production (Kubernetes)

```bash
# Create a backup job
cat <<EOF | kubectl apply -f -
apiVersion: batch/v1
kind: Job
metadata:
  name: db-backup-$(date +%Y%m%d-%H%M%S)
  namespace: app
spec:
  template:
    spec:
      containers:
      - name: backup
        image: alpine:latest
        command:
        - /bin/sh
        - -c
        - |
          apk add --no-cache sqlite
          # Assuming PVC is mounted at /data
          cp /data/*.db /backup/
          echo "Backup complete"
        volumeMounts:
        - name: database
          mountPath: /data
        - name: backup-storage
          mountPath: /backup
      volumes:
      - name: database
        persistentVolumeClaim:
          claimName: database-pvc
      - name: backup-storage
        persistentVolumeClaim:
          claimName: backup-pvc
      restartPolicy: Never
EOF

# Monitor backup job
kubectl get jobs -n app
kubectl logs job/db-backup-$(date +%Y%m%d-%H%M%S) -n app
```

### NATS JetStream

NATS JetStream maintains state on disk automatically. To backup:

```bash
# Backup JetStream data
kubectl cp app/nats-0:/data ./backups/nats-$(date +%Y%m%d_%H%M%S)
```

### Valkey (Redis)

```bash
# Trigger RDB snapshot
kubectl exec -n app valkey-0 -- valkey-cli BGSAVE

# Wait for completion
kubectl exec -n app valkey-0 -- valkey-cli LASTSAVE

# Backup RDB file
kubectl cp app/valkey-0:/data/dump.rdb ./backups/valkey-$(date +%Y%m%d_%H%M%S).rdb
```

### MinIO Object Storage

MinIO has built-in replication. To backup to external storage:

```bash
# Use mc (MinIO client) to sync to another location
mc alias set myminio http://localhost:9000 minioadmin minioadmin
mc mirror myminio/uploads ./backups/minio-uploads-$(date +%Y%m%d_%H%M%S)
```

## Restore Procedures

### libSQL/Turso Database

```bash
# Stop services
cargo run -p repo-tools -- infra local down

# Restore from backup
BACKUP_FILE="./backups/20260412_120000/app.db"
cp "$BACKUP_FILE" .data/app.db

# Restart services
cargo run -p repo-tools -- infra local up

# Verify restore
sqlite3 .data/app.db "SELECT COUNT(*) FROM sessions;"
```

### NATS JetStream

```bash
# Stop NATS
kubectl scale statefulset nats -n app --replicas=0

# Restore data
kubectl cp ./backups/nats-20260412_120000 app/nats-0:/data

# Restart NATS
kubectl scale statefulset nats -n app --replicas=1
```

### Valkey

```bash
# Restore RDB file
kubectl cp ./backups/valkey-20260412_120000.rdb app/valkey-0:/data/dump.rdb

# Restart Valkey
kubectl rollout restart statefulset valkey -n app
```

## Automated Backup Schedule

### CronJob for Daily Backups

```yaml
apiVersion: batch/v1
kind: CronJob
metadata:
  name: daily-backup
  namespace: app
spec:
  schedule: "0 2 * * *"  # Daily at 2 AM
  jobTemplate:
    spec:
      template:
        spec:
          containers:
          - name: backup
            image: alpine:latest
            command:
            - /bin/sh
            - -c
            - |
              # Backup database, NATS, Valkey
              echo "Backup started at $(date)"
              # ... backup commands ...
              echo "Backup completed at $(date)"
          restartPolicy: OnFailure
```

## Verification

After backup, always verify:

```bash
# Check backup files exist and are non-empty
ls -lh ./backups/

# Verify database backup integrity
sqlite3 ./backups/20260412_120000/app.db "PRAGMA integrity_check;"

# Test restore in a separate environment
# NEVER test restore in production!
```

## Troubleshooting

### Backup Fails

1. Check disk space: `df -h`
2. Check permissions: `ls -la ./backups/`
3. Check logs: `kubectl logs job/<backup-job>`

### Restore Fails

1. Verify backup file integrity
2. Check version compatibility
3. Review restore logs for errors
4. Try partial restore (specific tables)

## See Also

- [Migration Runner](../migrations/runner/README.md)
- [Health Checks](health-checks.md)
- [Incident Response](incident-response.md)
