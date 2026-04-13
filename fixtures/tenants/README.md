# Demo Tenants

> 多租户演示用种子数据。

## 需要创建的租户

| 租户 ID | 名称 | 用途 |
|--------|------|------|
| `demo-acme` | Acme Corp | 默认演示租户 |
| `demo-acme` 成员 | admin@acme.demo | 管理员用户 |
| `demo-acme` 成员 | user@acme.demo | 普通用户 |

## 格式

```yaml
# tenants/demo-acme.yaml
tenant_id: demo-acme
name: Acme Corp
plan: demo
features:
  - counter
  - agent
  - settings
  - chat
members:
  - email: admin@acme.demo
    role: admin
  - email: user@acme.demo
    role: member
```
