# K3s Cluster Deployment Guide

> Deploy the platform on a Kubernetes cluster using K3s.

## Prerequisites

| Requirement | Specification |
|-------------|---------------|
| Nodes | 1+ (staging), 3+ (production) |
| CPU | 2+ cores per node |
| RAM | 4+ GB per node |
| Storage | 50+ GB per node |
| OS | Ubuntu 22.04 LTS |
| kubectl | Installed and configured |
| Helm | Installed (for some addons) |

## Architecture

See [Deployment Diagram](../architecture/deployment/01-deployment.md) for detailed architecture.

## Step 1: Install K3s

### Single Node (Dev/Staging)

```bash
# On the server
curl -sfL https://get.k3s.io | sh -

# Get kubeconfig
sudo cp /etc/rancher/k3s/k3s.yaml ~/.kube/config
sudo chown $USER ~/.kube/config

# Verify
kubectl get nodes
```

### Multi-Node (Production)

```bash
# Server node 1
curl -sfL https://get.k3s.io | K3S_TOKEN=your-token sh -s server \
  --cluster-init \
  --tls-san your-domain.com

# Server node 2+
curl -sfL https://get.k3s.io | K3S_TOKEN=your-token sh -s server \
  --server https://node1-ip:6443 \
  --tls-san your-domain.com

# Worker nodes
curl -sfL https://get.k3s.io | K3S_URL=https://server-ip:6443 K3S_TOKEN=your-token sh -
```

## Step 2: Install Cilium CNI

```bash
# Install Cilium via Helm
helm repo add cilium https://helm.cilium.io/

helm install cilium cilium/cilium \
  --namespace kube-system \
  --set k8sServiceHost=your-server-ip \
  --set k8sServicePort=6443 \
  --set gatewayAPI.enabled=true \
  --set hubble.enabled=true \
  --set hubble.relay.enabled=true \
  --set hubble.ui.enabled=true
```

## Step 3: Apply Base Infrastructure

```bash
# Apply base manifests (RBAC, NetworkPolicy)
kubectl apply -k infra/k3s/base

# Apply infrastructure addons (NATS, Valkey, MinIO)
kubectl apply -k infra/kubernetes/addons

# Verify
kubectl get pods -n infrastructure
```

## Step 4: Deploy Applications

### Option A: Manual Deployment

```bash
# Build application images
docker build -f infra/images/Dockerfile.rust-service -t web-bff:latest .

# Load images into K3s (single node)
k3s ctr images import web-bff.tar

# Or push to registry
docker push your-registry/web-bff:latest

# Apply application manifests
kubectl apply -k infra/k3s/overlays/dev
```

### Option B: GitOps (Flux)

See [GitOps Guide](./gitops.md) for automated deployment.

## Step 5: Configure Gateway

```yaml
# gateway.yaml
apiVersion: gateway.networking.k8s.io/v1
kind: Gateway
metadata:
  name: main-gateway
  namespace: default
spec:
  gatewayClassName: cilium
  listeners:
  - name: http
    protocol: HTTP
    port: 80
  - name: https
    protocol: HTTPS
    port: 443
    tls:
      certificateRefs:
      - name: tls-cert
---
apiVersion: gateway.networking.k8s.io/v1
kind: HTTPRoute
metadata:
  name: web-bff-route
spec:
  parentRefs:
  - name: main-gateway
  rules:
  - matches:
    - path:
        type: PathPrefix
        value: /api
    backendRefs:
    - name: web-bff
      port: 3000
  - backendRefs:
    - name: web-bff
      port: 3000
```

```bash
kubectl apply -f gateway.yaml
```

## Step 6: Run Migrations

```bash
# Run migrations from a Job
kubectl apply -f - << EOF
apiVersion: batch/v1
kind: Job
metadata:
  name: migration
spec:
  template:
    spec:
      containers:
      - name: migrate
        image: your-registry/migration-runner:latest
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: database-secrets
              key: url
      restartPolicy: Never
  backoffLimit: 1
EOF

# Watch migration
kubectl logs -f job/migration
```

## Step 7: Verify

```bash
# Check all pods
kubectl get pods -A

# Check services
kubectl get svc

# Check gateway
kubectl get gateway

# Test endpoint
curl https://your-domain.com/healthz
```

## Scaling

### Horizontal Pod Autoscaling

```yaml
# hpa.yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: web-bff-hpa
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: web-bff
  minReplicas: 2
  maxReplicas: 10
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80
```

```bash
kubectl apply -f hpa.yaml
```

### Worker Scaling

Workers scale independently based on workload:

| Worker | Scaling Metric |
|--------|---------------|
| Indexer | NATS stream consumer lag |
| Outbox Relay | Outbox table row count (1 replica, leader elected) |
| Projector | Event processing lag |
| Scheduler | Fixed 1 replica |
| Sync Reconciler | Conflict count |

## Upgrading K3s

```bash
# On server nodes (one at a time)
curl -sfL https://get.k3s.io | sh -

# On worker nodes
curl -sfL https://get.k3s.io | K3S_URL=https://server-ip:6443 K3S_TOKEN=your-token sh -
```

## Troubleshooting

### Pod won't start
```bash
kubectl describe pod <pod-name>
kubectl logs <pod-name>
```

### Network issues
```bash
# Check NetworkPolicy
kubectl get networkpolicy

# Check Cilium
kubectl exec -n kube-system ds/cilium -- cilium status
```

### Storage issues
```bash
# Check PVCs
kubectl get pvc

# Check storage class
kubectl get storageclass
```
