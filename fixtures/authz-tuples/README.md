# Authorization Tuples

> OpenFGA 授权元数据种子。
> 当前授权模型未正式启用 OpenFGA，此处预留。

## 预期元组

| user | relation | object | 用途 |
|------|---------|--------|------|
| user:admin | owner | tenant:demo-acme | 租户拥有者 |
| user:alice | member | tenant:demo-acme | 租户成员 |
| user:admin | admin | service:admin | 管理员权限 |
