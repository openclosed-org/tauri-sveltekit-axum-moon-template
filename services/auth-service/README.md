# Auth Service

> 目的：标记 `auth-service` 当前只是语义保留位和实验骨架，避免它被误当成默认参考样例。

## 状态

- status: `stub`
- 角色：认证能力占位与实验骨架
- 说明：`model.yaml` 已保留 `auth-session` 与登录语义，但实现尚未收敛成默认参考链

## 责任

1. 为未来认证边界预留 service 名称与 ownership 位置。
2. 表达 `auth-session`、登录回调、session 查询等语义草图。
3. 承载当前认证相关实验代码，而不是提供稳定模板。

## 入口

1. `model.yaml`：当前 auth 语义保留位。
2. `src/`：现有实验/遗留骨架代码。
3. `migrations/`：与当前 crate 一起保留的本地存储脚手架。

## 验证

```bash
cargo check -p auth-service
cargo test -p auth-service
```

## 不要这样用

1. 不要把这个目录当成新 service 的 copy target。
2. 不要把 `model.yaml` 里的保留语义误解为已完成的生产能力。
3. 不要在没有补齐 contracts、server、worker、secrets 链路前，把它写成已接入默认参考链。
