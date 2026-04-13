# Demo Users

> 用户域种子数据。

## 需要创建的用户

| 用户 ID | 邮箱 | 角色 | 用途 |
|--------|------|------|------|
| `user-alice` | alice@example.com | 标准用户 | 功能测试 |
| `user-bob` | bob@example.com | 标准用户 | 功能测试 |
| `user-admin` | admin@example.com | 管理员 | 管理端测试 |

## 格式

```yaml
# users/alice.yaml
user_id: user-alice
email: alice@example.com
display_name: Alice
oauth_provider: demo
created_at: "2026-01-01T00:00:00Z"
```
