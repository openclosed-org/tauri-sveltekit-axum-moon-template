# 📋 新增 Feature 完整开发路径规范

> **适用场景**: 任何新业务模块从零到上线的完整流程。
> **目标**: 新成员或 Agent 拿到需求后，无需询问，按此文一步步执行。

---

## 0. 前提：需求确认

在写任何代码之前，明确：

| 问题 | 示例回答 |
|------|---------|
| 这个 feature 解决什么问题？ | "用户需要保存 LLM API 连接配置" |
| 用户是谁？（用户级 / 租户级 / 平台级） | "用户级 — 按 user_sub 隔离" |
| 需要持久化吗？ | "是 — 存到 Turso/SQLite" |
| 需要实时/流式吗？ | "否 — CRUD 即可" |
| 哪些端需要消费？（Web / Tauri / Mobile） | "Web + Tauri" |
| 与其他 service 有依赖吗？ | "无 — 独立配置" |

---

## 1. 定义契约（packages/contracts/）

**文件**: `packages/contracts/api/src/lib.rs`

**规则**:
- DTO 必须同时标注 `#[derive(ToSchema, TS)]`（utoipa OpenAPI + ts-rs 前端类型）
- 字段使用 snake_case
- 敏感字段（API Key 等）在 DTO 层做脱敏命名（`api_key_masked`）

```rust
/// Agent API 连接配置
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS)]
#[ts(export, export_to = "settings/")]
pub struct AgentConnectionResponse {
    pub api_key_masked: String,
    pub base_url: String,
    pub model: String,
}

/// 更新 Agent 连接配置请求
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS)]
#[ts(export, export_to = "settings/")]
pub struct UpdateAgentConnectionRequest {
    pub api_key: String,
    pub base_url: Option<String>,
    pub model: Option<String>,
}
```

**验证**: `cargo check -p contracts-api`

---

## 2. 定义 Feature Trait（packages/features/）

**文件**: `packages/features/<domain>/src/lib.rs`

**规则**:
- 只定义 trait + error，**不得包含实现**
- 不得依赖 `services/` 或 `usecases/`
- error 使用 `thiserror`

```rust
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConnectionSettings {
    pub api_key: String,
    pub base_url: String,
    pub model: String,
}

#[async_trait]
pub trait SettingsService: Send + Sync {
    async fn get_settings(&self, user_sub: &str) -> Result<AgentConnectionSettings, SettingsError>;
    async fn update_agent_connection(
        &self,
        user_sub: &str,
        settings: AgentConnectionSettings,
    ) -> Result<AgentConnectionSettings, SettingsError>;
}

#[derive(Debug, thiserror::Error)]
pub enum SettingsError {
    #[error("Database error: {0}")]
    Database(String),
    #[error("Settings not found: {0}")]
    NotFound(String),
    #[error("Validation error: {0}")]
    Validation(String),
}
```

**验证**: `cargo check -p feature-settings`

---

## 3. 创建 Service 骨架

```bash
cp -r services/counter-service services/<domain>-service
cd services/<domain>-service
# 全局替换 counter → <domain>
find . -type f -name '*.rs' -exec sed -i '' 's/counter/<domain>/g' {} +
find . -type f -name '*.toml' -exec sed -i '' 's/counter/<domain>/g' {} +
find . -type f -name '*.sql' -exec sed -i '' 's/counter/<domain>/g' {} +
```

注册到根 `Cargo.toml`:
```toml
[workspace]
members = [
    ...
    "services/<domain>-service",
]

[workspace.dependencies]
<domain>-service = { path = "services/<domain>-service" }
```

---

## 4. 实现 Domain 层

**文件**: `services/<domain>-service/src/domain/entity.rs`

**规则**:
- 零外部依赖（只能依赖 serde/chrono/thiserror）
- 不可变风格优先（返回新实例而非修改）
- 领域不变量在构造方法中校验

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSettings {
    pub user_sub: String,                    // 主键
    pub agent_connection: AgentConnectionSettings,
    pub updated_at: String,
}
```

**文件**: `services/<domain>-service/src/domain/error.rs`

```rust
#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    #[error("Not found: {0}")]
    NotFound(String),
}
```

---

## 5. 实现 Ports 层

**文件**: `services/<domain>-service/src/ports/mod.rs`

```rust
use async_trait::async_trait;

pub type RepositoryError = Box<dyn std::error::Error + Send + Sync>;

#[async_trait]
pub trait SettingsRepository: Send + Sync {
    async fn get_or_create(&self, user_sub: &str) -> Result<UserSettings, RepositoryError>;
    async fn update_agent_connection(
        &self,
        user_sub: &str,
        settings: AgentConnectionSettings,
    ) -> Result<UserSettings, RepositoryError>;
}
```

---

## 6. 实现 Application 层

**文件**: `services/<domain>-service/src/application/service.rs`

```rust
pub struct ApplicationSettingsService<R: SettingsRepository> {
    repo: R,
}

impl<R: SettingsRepository> ApplicationSettingsService<R> {
    pub fn new(repo: R) -> Self { Self { repo } }
}

#[async_trait]
impl feature_settings::SettingsService for ApplicationSettingsService<R> {
    async fn get_settings(&self, user_sub: &str) -> Result<AgentConnectionSettings, SettingsError> {
        self.repo.get_or_create(user_sub)
            .await
            .map(|s| s.agent_connection)
            .map_err(|e| SettingsError::Database(e.to_string()))
    }

    async fn update_agent_connection(...) -> Result<...> { ... }
}

pub const MIGRATION: &str = "CREATE TABLE IF NOT EXISTS settings (...)";
```

---

## 7. 实现 Infrastructure 层

**文件**: `services/<domain>-service/src/infrastructure/libsql_adapter.rs`

```rust
pub struct LibSqlSettingsRepository<P: LibSqlPort> {
    port: P,
}

#[async_trait]
impl<P: LibSqlPort> SettingsRepository for LibSqlSettingsRepository<P> {
    async fn get_or_create(&self, user_sub: &str) -> Result<UserSettings, RepositoryError> {
        let rows: Vec<SettingsRow> = self.port
            .query("SELECT ... FROM settings WHERE user_sub = ?", vec![...])
            .await?;
        // ... 行转实体
    }
}
```

---

## 8. 编写 Tests + Migrations

### 单元测试

**文件**: `services/<domain>-service/tests/unit/<domain>_service_test.rs`

**规则**:
- Mock repository（内存 HashMap），**不碰数据库**
- 至少覆盖：正常路径、边界条件、隔离性（多用户/租户不泄露）

```rust
struct MockSettingsRepository { store: Arc<Mutex<Vec<UserSettings>>> }

#[async_trait]
impl SettingsRepository for MockSettingsRepository { ... }

#[tokio::test]
async fn user_isolation_settings_dont_leak() { ... }
```

### 迁移

**文件**: `services/<domain>-service/migrations/001_create_<domain>.sql`

```sql
CREATE TABLE IF NOT EXISTS settings (
    user_sub TEXT PRIMARY KEY,
    api_key TEXT NOT NULL DEFAULT '',
    base_url TEXT NOT NULL DEFAULT 'https://api.openai.com/v1',
    model TEXT NOT NULL DEFAULT 'gpt-4o-mini',
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
```

---

## 9. 注册 HTTP 路由

### servers/api

**文件**: `servers/api/src/routes/<domain>.rs`

```rust
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/<domain>", get(get_<domain>))
        .route("/api/<domain>", put(update_<domain>))
}
```

注册到 `servers/api/src/routes/mod.rs`:
```rust
pub mod <domain>;
// 在 api_router() 中:
.merge(<domain>::router())
```

注册迁移到 `servers/api/src/state.rs`:
```rust
let settings_repo = <domain>_service::infrastructure::LibSql<...>Repository::new(embedded_db);
settings_repo.migrate().await?;
```

### servers/bff

**文件**: `servers/bff/web-bff/src/handlers/<domain>.rs`

结构与 servers/api 相同，唯一区别：
- 使用 `BffState` 而非 `AppState`
- 在 `handlers/mod.rs` 中注册模块
- 在 `lib.rs` 的 `api_routes` 中 merge

### Cargo.toml 更新

```toml
# servers/api/Cargo.toml
<domain>-service = { workspace = true }
feature-<domain> = { workspace = true }

# servers/bff/web-bff/Cargo.toml
<domain>-service = { workspace = true }
```

---

## 10. Tauri Commands

**文件**: `packages/adapters/hosts/tauri/src/commands/<domain>.rs`

```rust
#[tauri::command]
pub async fn <domain>_get(app: AppHandle) -> Result<serde_json::Value, String> {
    let db = app.state::<EmbeddedTurso>().inner().clone();
    let repo = <domain>_service::infrastructure::LibSql<...>Repository::new(db);
    let service = <domain>_service::application::Application<...>Service::new(repo);
    // ...
}
```

注册到 `packages/adapters/hosts/tauri/src/commands/mod.rs`:
```rust
pub mod <domain>;
// 在 generate_handler! 中添加:
<domain>::<domain>_get,
<domain>::<domain>_update,
```

更新 `Cargo.toml`:
```toml
<domain>-service = { workspace = true }
```

---

## 11. 前端消费

**文件**: `apps/web/src/routes/<domain>/+page.svelte`

```svelte
<script lang="ts">
// 检测运行环境
const isTauri = (window as any).__TAURI__ !== undefined;

async function loadSettings() {
  if (isTauri) {
    return await __TAURI__.core.invoke('<domain>_get');
  } else {
    const res = await fetch('/api/<domain>', {
      headers: { 'Authorization': `Bearer ${token}` }
    });
    return res.json();
  }
}
</script>
```

---

## 12. E2E 测试

**文件**: `apps/web/tests/e2e/<domain>.test.ts`

```typescript
test('settings page loads and saves', async ({ page }) => {
  await page.goto('/<domain>');
  await expect(page.getByText('Settings')).toBeVisible();
  // ... fill, save, verify
});
```

---

## 13. 验证清单

在提交 PR 之前，确认：

- [ ] `cargo check --workspace` 零错误
- [ ] `cargo test -p <domain>-service` 全部通过
- [ ] `cargo build -p runtime_server` 通过
- [ ] `cargo build -p web-bff` 通过
- [ ] `cargo build -p runtime_tauri` 通过（如果添加了 Tauri commands）
- [ ] 新增的 migration 在 `state.rs` 中注册
- [ ] OpenAPI doc 中注册了新路由（`servers/api/src/lib.rs`）
- [ ] README.md 准确描述模块用途
- [ ] 无 `usecases::` 引用（历史已清除）

---

## 附录：快速参考

### 隔离维度选择

| 场景 | 隔离键 | 示例 |
|------|--------|------|
| 租户级配置/数据 | `tenant_id` | counter, tenant members |
| 用户级偏好 | `user_sub` | settings, theme |
| 平台级数据 | 无隔离 | app version, feature flags |

### 命名约定

| 层级 | 前缀 | 示例 |
|------|------|------|
| Service crate | `<domain>-service` | `settings-service` |
| Feature crate | `feature-<domain>` | `feature-settings` |
| Repository trait | `<Domain>Repository` | `SettingsRepository` |
| Service trait | `<Domain>Service` | `SettingsService` |
| Application service | `Application<Domain>Service` | `ApplicationSettingsService` |
| Infrastructure adapter | `LibSql<Domain>Repository` | `LibSqlSettingsRepository` |

### 常见错误

1. ❌ 在 domain/ 中引入 async —— domain 层必须是同步的
2. ❌ service 之间直接 import —— 必须通过 contracts/events
3. ❌ 在 servers/ 中写业务逻辑 —— servers 只做组合/协议转换
4. ❌ 忘记注册 migration —— 运行时表不存在
5. ❌ DTO 忘记 `#[ts(export)]` —— 前端类型不生成
