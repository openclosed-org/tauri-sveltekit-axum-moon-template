# User Service

> 目的：标记 `user-service` 当前只是 identity/profile 语义占位，而不是默认参考 service。

## 状态

- status: `stub`
- 角色：用户身份与 profile 语义保留位
- 说明：当前实现仍以骨架为主，目录存在的意义大于代码成熟度

## 责任

1. 为 `user-profile` ownership 与查询语义保留稳定目录边界。
2. 表达用户初始化、用户与租户关系等早期语义草图。
3. 为后续 identity 相关能力留出 service 槽位。

## 入口

1. `model.yaml`：当前 `user-profile` 相关语义真理源。
2. `src/`：现有骨架实现。
3. `migrations/`：与本地状态相关的脚手架。

## 验证

```bash
cargo check -p user-service
cargo test -p user-service
```

## 不要这样用

1. 不要把它当成可复制的 reference service。
2. 不要把目录存在解读为 identity 边界已经收敛完成。
3. 不要绕过 `services/README.md` 的目录级分类，把这里的骨架代码升级成默认模式。
