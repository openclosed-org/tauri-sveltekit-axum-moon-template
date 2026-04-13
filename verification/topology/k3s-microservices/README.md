# Topology: K3s Microservices

> K3s 微服务拓扑验证测试。

## 测试场景

1. 所有服务在 K3s 集群中启动
2. 服务间通过 NATS 通信
3. BFF 能正确路由到后端服务
4. Workers 能消费消息并执行
5. 健康检查全部通过

## 实现状态

⚠️ 待实现。当前 `verification/topology/single-vps/` 有占位测试。
