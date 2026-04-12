# Tenant — 多租户域

> 租户实体、成员管理、隔离策略。
> 与 user-service 协作完成 OAuth 回调后的租户初始化流程。

```bash
cargo build -p tenant-service
```

当前状态：业务实现在 `packages/core/usecases/tenant_service.rs`，待迁移至本目录。
架构说明见 [services/README.md](../README.md)。
