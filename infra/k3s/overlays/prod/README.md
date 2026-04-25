# Prod Overlay

> 生产环境 overlay 目标形状。当前保留了资源与副本策略意图，但不应被写成已完整验证的生产事实。

## 当前事实

1. 包含副本数与资源限制 patch。
2. 目录中存在 web 侧 HPA 清单，但仍需与 base 资源命名和实际交付链继续校准。
3. 这里的多副本/HPA 不能替代 worker shared checkpoint、dedupe、lease 等可靠性前提。

## 使用

```bash
kubectl apply -k infra/k3s/overlays/prod
```
