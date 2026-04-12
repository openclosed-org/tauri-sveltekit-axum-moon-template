# User — 用户域

> 用户实体、OAuth 身份验证、用户-租户绑定关系。
> 处理用户首次登录时的租户自动初始化流程。

```bash
cargo test -p user-service
cargo build -p user-service
```

注意：此服务的业务逻辑已通过 HTTP 路由暴露，但尚无专属 user 路由文件（租户初始化通过 tenant 路由触发）。
架构说明见 [services/README.md](../README.md)。
