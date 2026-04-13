# Tenant — 多租户域

> 租户实体、成员管理、隔离策略。
> 与 user-service 协作完成 OAuth 回调后的租户初始化流程。

```bash
cargo build -p tenant-service
cargo test -p tenant-service
```

当前状态：✅ 已迁移完成。业务逻辑完整实现在本目录的 domain/application/infrastructure/ports 中。
