# Staging Overlay

> 预发环境 overlay，占位意义大于当前验证完成度。

## 当前事实

1. 提供独立的 host 与资源 patch。
2. 仍有与 base 命名、资源文件一致性相关的收敛工作待做。
3. 不应被写成“已经生产等价验证完成”的事实。

## 使用

```bash
kubectl apply -k infra/k3s/overlays/staging
```
