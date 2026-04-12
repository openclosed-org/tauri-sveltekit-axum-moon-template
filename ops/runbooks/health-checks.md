# Health Checks Runbook

This runbook covers health check procedures for verifying system health.

## Health Check Endpoints

| Service | Health Endpoint | Expected Response |
|---------|----------------|-------------------|
| Gateway | `GET /healthz` | `200 OK` |
| API | `GET /healthz` | `{"status": "ok"}` |
| Web | `GET /healthz` | `200 OK` |
| NATS | `GET http://localhost:8222/healthz` | `{"status": "ok"}` |
| Valkey | `redis-cli ping` | `PONG` |
| MinIO | `mc ready local` | Success exit code |

## Local Development Health Checks

### Quick Health Check Script

```bash
#!/usr/bin/env bash
# Quick health check for all local services

echo "Checking service health..."

# Check NATS
if curl -sf http://localhost:8222/healthz > /dev/null; then
    echo "✅ NATS: Healthy"
else
    echo "❌ NATS: Unhealthy"
fi

# Check Valkey
if redis-cli -h localhost -p 6379 ping | grep -q PONG; then
    echo "✅ Valkey: Healthy"
else
    echo "❌ Valkey: Unhealthy"
fi

# Check MinIO
if docker exec $(docker ps -q -f name=minio) mc ready local 2>/dev/null; then
    echo "✅ MinIO: Healthy"
else
    echo "❌ MinIO: Unhealthy"
fi

# Check API (if running)
if curl -sf http://localhost:3001/healthz > /dev/null; then
    echo "✅ API: Healthy"
else
    echo "⚠️  API: Not running or unhealthy"
fi

# Check Gateway
if curl -sf http://localhost:3000/healthz > /dev/null; then
    echo "✅ Gateway: Healthy"
else
    echo "⚠️  Gateway: Not running or unhealthy"
fi
```

### Manual Health Checks

```bash
# Gateway
curl -v http://localhost:3000/healthz

# API
curl -v http://localhost:3001/healthz

# NATS monitoring
curl -v http://localhost:8222/healthz

# Valkey
redis-cli -h localhost -p 6379 ping

# MinIO API
curl -v http://localhost:9000/minio/health/live

# MinIO Console
curl -v http://localhost:9001
```

## Kubernetes Health Checks

### Check Pod Status

```bash
# Get all pods in app namespace
kubectl get pods -n app

# Check pod details
kubectl describe pod <pod-name> -n app

# Check pod logs
kubectl logs <pod-name> -n app

# Check pod events
kubectl get events -n app --sort-by='.lastTimestamp'
```

### Check Service Health

```bash
# Check services
kubectl get services -n app

# Check endpoints
kubectl get endpoints -n app

# Port-forward to test locally
kubectl port-forward -n app svc/api 3001:3001
curl http://localhost:3001/healthz
```

### Check Deployments

```bash
# Check deployment status
kubectl get deployments -n app

# Check rollout status
kubectl rollout status deployment/api -n app

# Check replica status
kubectl get pods -n app -o wide
```

## Database Health Checks

### libSQL/Turso

```bash
# Check database connectivity
sqlite3 .data/app.db "SELECT 1;"

# Check table integrity
sqlite3 .data/app.db "PRAGMA integrity_check;"

# Check table sizes
sqlite3 .data/app.db "SELECT name, COUNT(*) FROM sqlite_master WHERE type='table' GROUP BY name;"

# Check recent sessions
sqlite3 .data/app.db "SELECT COUNT(*) FROM sessions WHERE expires_at > datetime('now');"
```

### NATS JetStream

```bash
# Check JetStream status
curl -s http://localhost:8222/varz | jq .jetstream

# Check streams
curl -s http://localhost:8222/jsz | jq .streams

# Check consumers
curl -s http://localhost:8222/jsz?streams=EVENTS | jq .streams[0].consumers
```

### Valkey

```bash
# Check memory usage
redis-cli -h localhost -p 6379 INFO memory

# Check connected clients
redis-cli -h localhost -p 6379 INFO clients

# Check keyspace
redis-cli -h localhost -p 6379 INFO keyspace

# Test set/get
redis-cli -h localhost -p 6379 SET test:health "ok"
redis-cli -h localhost -p 6379 GET test:health
redis-cli -h localhost -p 6379 DEL test:health
```

## Application-Level Health Checks

### Authentication Service

```bash
# Test auth health check
curl -X GET http://localhost:3001/api/health

# Test token endpoint
curl -X POST http://localhost:3001/api/auth/token \
  -H "Content-Type: application/json" \
  -d '{"user_id": "test", "user_sub": "test-sub"}'
```

### User Service

```bash
# Test user profile endpoint
curl -X GET http://localhost:3001/api/users/me \
  -H "Authorization: Bearer <token>"
```

### Tenant Service

```bash
# Test tenant initialization
curl -X POST http://localhost:3001/api/tenants/init \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <token>" \
  -d '{"user_sub": "test-sub", "user_name": "Test User"}'
```

## Monitoring & Alerting

### Prometheus Metrics (if configured)

```bash
# Check metrics endpoint
curl http://localhost:3001/metrics

# Query specific metrics
curl http://localhost:3001/metrics | grep http_requests_total
```

### Log Monitoring

```bash
# Follow API logs
kubectl logs -f deployment/api -n app

# Search for errors
kubectl logs deployment/api -n app | grep ERROR

# Check recent logs
kubectl logs --tail=100 deployment/api -n app
```

## Troubleshooting Unhealthy Services

### Step 1: Check Pod Events

```bash
kubectl describe pod <pod-name> -n app
```

Look for:
- Failed scheduling
- Image pull errors
- CrashLoopBackOff
- OOMKilled

### Step 2: Check Container Logs

```bash
kubectl logs <pod-name> -n app --previous  # If crashed
kubectl logs <pod-name> -n app
```

### Step 3: Check Resource Usage

```bash
kubectl top pods -n app
kubectl top nodes
```

### Step 4: Restart Pod

```bash
kubectl rollout restart deployment/<name> -n app
kubectl delete pod <pod-name> -n app  # Will be recreated
```

## Regular Health Check Schedule

| Check | Frequency | Method |
|-------|-----------|--------|
| Pod status | Continuous | Kubernetes liveness probe |
| API health | Every 1 min | HTTP healthz endpoint |
| Database connectivity | Every 5 min | Simple query test |
| Disk usage | Every 1 hour | `df -h` check |
| Memory usage | Every 1 hour | `free -m` check |
| Backup status | Daily | Verify backup jobs |

## See Also

- [Backup & Restore](backup-restore.md)
- [Incident Response](incident-response.md)
- [Service Deployment](service-deployment.md)
