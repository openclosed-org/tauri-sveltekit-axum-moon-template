# Authorization Tuples — OpenFGA / MockAuthz 种子数据

> Phase 5 当前已补齐仓库内集成面：`counter` 路径会真实调用 `AuthzPort`，`web-bff` 可在 `MockAuthzAdapter` 与 `OpenFGA` adapter 间切换。
> 当 OpenFGA 实例部署后，这些元组可通过 `write_tuple` API 写入 OpenFGA store；在 dev/test 中，`tenant/init` 也会向当前 authz adapter 写入最小授权元组。

## 授权模型

见 `packages/authz/src/model.rs` 中的 `AuthorizationModel::default_counter_model()`。

类型:
- `user` — 认证身份
- `tenant` — 多租户边界
- `counter` — 租户下的计数器资源

## 默认种子元组（dev/test）

| user | relation | object | 用途 |
|------|---------|--------|------|
| user:dev-test-user | owner | tenant:dev-tenant-001 | dev 模式租户拥有者 |
| user:dev-test-user | member | tenant:dev-tenant-001 | dev 模式租户成员 |
| user:dev-test-user | can_write | counter:dev-tenant-001 | 可以操作 counter |
| user:dev-test-user | can_read | counter:dev-tenant-001 | 可以读取 counter |

## 使用方式

```rust
// 在测试或 dev 初始化中
use authz::{MockAuthzAdapter, AuthzTupleKey};

let authz = MockAuthzAdapter::new();
authz.seed(vec![
    AuthzTupleKey::new("user:dev-test-user", "owner", "tenant:dev-tenant-001"),
    AuthzTupleKey::new("user:dev-test-user", "can_write", "counter:dev-tenant-001"),
]).await;
```

## 注意

- MockAuthzAdapter 在 store 为空时默认 **allow-all**（dev 模式便利）。
- 一旦 seed 了元组，就进入 **strict** 模式，只有显式匹配的元组才通过。
- 这确保了 dev 和 prod 的行为一致性（prod 的 OpenFGA 不会 allow-all）。
