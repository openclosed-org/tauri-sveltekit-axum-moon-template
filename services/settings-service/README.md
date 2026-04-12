# Settings — 用户级配置服务

> 用户偏好设置（API Key、Base URL、Model）。**用户级**（按 user_sub），非租户级。

```bash
cargo test -p settings-service
cargo build -p settings-service
```

与 counter-service 的关键差异：counter 按 tenant_id 隔离，settings 按 user_sub 隔离。
