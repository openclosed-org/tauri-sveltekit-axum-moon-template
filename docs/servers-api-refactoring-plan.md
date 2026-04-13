# servers/api 重构计划

> **生成时间**: 2026-04-12
> **核心目标**: 将 `servers/api` 单体 API 服务器重构为符合架构规范的多 BFF 拓扑
> **原则**: 只分析不修改，给出完整可执行方案

---

## 1. 现状诊断

### 1.1 `servers/api` 是什么

`servers/api` 是当前项目的**单一 API 入口**，一个 Axum HTTP 服务器（默认端口 3001），聚合了所有业务域的端点：

| 业务域 | 端点 | 状态 |
|--------|------|------|
| Health | `/healthz`, `/readyz` | ✅ 正常工作 |
| Tenant | `/api/tenant/init` | ⚠️ 直接写 SQL，绕过 tenant-service |
| Counter | `/api/counter/*` (4 endpoints) | ⚠️ 每次请求构造 service |
| Admin | `/api/admin/stats` | ⚠️ 唯一使用 adapter 模式的 handler |
| Agent | `/api/agent/*` (4 endpoints, SSE) | ⚠️ 直接构造 repository |
| Settings | `/api/settings` (GET/PUT) | ⚠️ 直接构造 repository + service |
| User | `/api/user/*` (2 endpoints) | ⚠️ 直接构造 repository |

### 1.2 架构违规清单

| # | 违规 | 严重性 | 违反规则 |
|---|------|--------|---------|
| 1 | **单体服务器而非独立 BFF** — 所有域在一个进程内 | 🔴 高 | codemap.yml 定义了 web-bff、admin-bff、gateway 为独立 server |
| 2 | **Handler 直接操作数据库** — tenant init 执行原始 SQL | 🔴 高 | repo-layout.md §3.5: "禁止直接操作数据库" |
| 3 | **Handler 构造 service/repository** — 每次请求 inline 实例化 | 🟡 中 | repo-layout.md §3.5: "handler 只负责解析请求、调用 application API、组织响应" |
| 4 | **越层依赖** — handler 直接 `use` service 的 `infrastructure` 层 | 🟡 中 | 应通过 ports 抽象，在 composition root 注入 |
| 5 | **错误类型知道具体数据库** — `AppError::DatabaseDirect(surrealdb::Error)` | 🟡 中 | vendor 只能进 adapters |
| 6 | **缺少 `handlers/` 目录** — 只有 `routes/` | 🟢 低 | codemap.yml server 模板要求 `handlers/` |
| 7 | **缺少 `README.md`** | 🟢 低 | codemap.yml required_files |

### 1.3 目标 BFF 当前状态

| 目录 | 状态 | 端口 | 关键特征 |
|------|------|------|---------|
| `servers/bff/web-bff/` | ✅ 功能完整 | 3010 | 嵌入式 Turso DB、JWT 中间件、全量 handler |
| `servers/bff/admin-bff/` | ⚠️ 部分实现 | 3020 | 无嵌入式 DB、占位端点、硬编码 URL、auth 未接线 |
| `servers/gateway/` | ✅ 功能骨架 | 3000 | Pingora 反向代理、健康检查、路由到 api/web |

### 1.4 服务完整度

| 服务 | 完整度 | 缺失 |
|------|--------|------|
| `counter-service` | ✅ 100% | 无 |
| `user-service` | ⚠️ 83% | `src/policies/` |
| `tenant-service` | ⚠️ 83% | `src/policies/` |
| `settings-service` | ⚠️ 67% | `src/policies/`, `src/events/` |
| `agent-service` | ⚠️ 67% | `src/policies/`, `src/events/` |
| `admin-service` | ⚠️ 50% | `src/policies/`, `src/events/`, `src/contracts/` |
| `event-bus` | ❌ 20% | 结构不同（基础设施库，非 domain service） |

---

## 2. 重构目标

### 2.1 最终态

```
servers/api/          →  删除（能力已迁移）
servers/bff/web-bff/  →  完整 Web BFF（已有）
servers/bff/admin-bff/→  补全 Admin BFF
servers/gateway/      →  完善 Gateway（已有基础）
```

### 2.2 职责分配

| 端点 | 当前在 | 应迁移到 | 理由 |
|------|--------|---------|------|
| `/healthz`, `/readyz` | api | web-bff (已有) | 健康检查 |
| `/api/tenant/init` | api | web-bff (已有) | 租户初始化 |
| `/api/counter/*` | api | web-bff (已有) | 计数器操作 |
| `/api/user/*` | api | web-bff (已有) | 用户端点 |
| `/api/settings` | api | web-bff (已有) | 设置 |
| `/api/agent/*` | api | web-bff (已有) | Agent 聊天 |
| `/api/admin/stats` | api | admin-bff | 管理仪表盘 |
| — | 不存在 | admin-bff 新增 | `/api/admin/tenants` |
| — | 不存在 | admin-bff 新增 | `/api/admin/metrics` |

### 2.3 关键发现

**web-bff 已经实现了 servers/api 的所有端点！** 对比两个实现：

| 端点 | servers/api | servers/bff/web-bff | 状态 |
|------|-------------|---------------------|------|
| `/healthz` | ✅ | ✅ | 等价 |
| `/readyz` | ✅ | ✅ | 等价 |
| `/api/tenant/init` | ✅ raw SQL | ✅ raw SQL | 等价（都有 raw SQL 问题） |
| `/api/counter/*` | ✅ inline | ✅ via AppState | web-bff 更规范 |
| `/api/user/*` | ✅ inline | ✅ via AppState | web-bff 更规范 |
| `/api/settings` | ✅ inline | ✅ via AppState | web-bff 更规范 |
| `/api/agent/*` | ✅ inline | ✅ via AppState | web-bff 更规范 |
| `/api/admin/stats` | ✅ | ✅ | 等价 |

**结论**: `servers/api` 是 web-bff 的**前身/原型实现**。web-bff 已经是更规范的版本（service 通过 AppState 注入而非每次请求构造）。

---

## 3. 执行计划

### Phase 0: 修复现有问题（必须先做）

在删除 `servers/api` 之前，需要确保 web-bff 和 admin-bff 没有遗漏。

#### 0.1 修复 web-bff 中 tenant/init 的 raw SQL 问题

**当前**: `servers/bff/web-bff/src/handlers/tenant.rs` 直接执行原始 SQL 创建 tenant 和 user_tenant 绑定。
**应改为**: 调用 `tenant-service` 的 application layer 方法。

**涉及文件**:
- `servers/bff/web-bff/src/handlers/tenant.rs`
- `services/tenant-service/src/application/` (需要添加 `init_tenant` 方法)

**工作量**: 小

#### 0.2 补全 admin-bff

**当前问题**:
1. `/api/admin/dashboard` 硬编码 `http://127.0.0.1:3001`（指向 servers/api）
2. `/api/admin/tenants` 返回空列表（占位）
3. `/api/admin/metrics` 返回零值（占位）
4. Auth 中间件未接线到路由
5. 无测试

**修复方案**:

| # | 操作 | 文件 | 说明 |
|---|------|------|------|
| 0.2.1 | 将 dashboard 改为直接调用 service | `handlers/dashboard.rs` | 不再 HTTP 调用内部 API，改为 composition root 注入 admin-service |
| 0.2.2 | 实现 tenants 端点 | 新增 `handlers/tenants.rs` | 调用 tenant-service |
| 0.2.3 | 实现 metrics 端点或删除 | `handlers/metrics.rs` | 暂时删除占位端点，或接入 observability |
| 0.2.4 | 接线 auth 中间件 | `lib.rs` | 将 `admin_tenant_middleware` 应用到需要认证的的路由 |
| 0.2.5 | 添加基本测试 | 新增 `tests/` 内容 | health endpoint 测试 |

**工作量**: 中

#### 0.3 补全 services 缺失的 policies/ 目录

对以下服务创建 `src/policies/` 目录（含 `.gitkeep` 和模块声明）：

| 服务 | 操作 |
|------|------|
| `user-service` | 创建 `src/policies/mod.rs` + `.gitkeep` |
| `tenant-service` | 创建 `src/policies/mod.rs` + `.gitkeep` |
| `settings-service` | 创建 `src/policies/mod.rs` + `.gitkeep` |
| `agent-service` | 创建 `src/policies/mod.rs` + `.gitkeep` |
| `admin-service` | 创建 `src/policies/mod.rs` + `.gitkeep` |

同时补齐 admin-service 缺失的 `src/events/` 和 `src/contracts/`。

**工作量**: 小

---

### Phase 1: 能力迁移验证

#### 1.1 验证 web-bff 完全覆盖 servers/api 功能

| 验证项 | 方法 |
|--------|------|
| 端点覆盖 | 对比 openapi.yaml 与 web-bff 路由注册 |
| 行为等价 | 对同一输入，比较 api 和 web-bff 的响应形状 |
| 中间件等价 | 比较 CORS、timeout、tracing、JWT 配置 |
| 数据库等价 | 确认使用同一嵌入式 Turso DB 路径 |

#### 1.2 更新基础设施配置

| 文件 | 当前引用 servers/api | 应改为 |
|------|---------------------|--------|
| `infra/local/docker-compose.yml` | `servers/api` 容器 | 改为 `web-bff` + `admin-bff` + `gateway` |
| `infra/kubernetes/` manifests | api deployment | web-bff + admin-bff deployments |
| `platform/model/deployables/` | api deployable | web-bff + admin-bff deployables |
| `Justfile` | `dev-api` 命令 | 确认已有 `dev-web-bff` 等 |
| 根 `Cargo.toml` | workspace members | 保持不变（已注册） |

#### 1.3 更新文档

| 文件 | 操作 |
|------|------|
| `docs/operations/` | 更新运维指南，删除 api 相关，新增 BFF 相关 |
| `docs/contracts/` | 更新契约文档，明确 BFF 路由映射 |
| `agent/codemap.yml` | 确认 servers 部分定义准确 |

---

### Phase 2: 删除 servers/api

#### 2.1 保存有用资产

`servers/api` 中值得保留的内容：

| 资产 | 当前位置 | 是否需要保留 | 说明 |
|------|---------|-------------|------|
| `openapi.yaml` | `servers/api/openapi.yaml` | ✅ 保留 | 移入 `docs/contracts/api-routes.yaml` 或 web-bff |
| `tests/` | `servers/api/tests/` | ⚠️ 审查 | 有值的测试移入 web-bff tests |
| `h3_server.rs` | `servers/api/src/h3_server.rs` | ❌ 不保留 | HTTP/3 骨架代码，未实现功能 |
| `config.rs` 模板 | `servers/api/src/config.rs` | ❌ 不保留 | web-bff 已有更好的版本 |

#### 2.2 执行删除

```
2.1 将 openapi.yaml 移至 docs/contracts/api-routes.yaml
2.2 审查 tests/，将有价值的测试移入 web-bff tests/
2.3 从根 Cargo.toml workspace members 移除 "servers/api"
2.4 从根 Cargo.toml [workspace.dependencies] 移除 runtime_server 引用
2.5 删除 servers/api/ 整个目录
2.6 更新 infra 配置中所有引用 servers/api 的地方
```

---

### Phase 3: 完善 Gateway

#### 3.1 当前状态

`servers/gateway/` 使用 Pingora 实现反向代理，已有基本路由逻辑：
- `/api/*` → API upstream (3001)
- 其他 → Web upstream (3002)

#### 3.2 需要的改进

| # | 改进 | 优先级 | 说明 |
|---|------|--------|------|
| 3.1 | 更新 upstream 端口 | 🔴 高 | API upstream 从 3001 改为 3010 (web-bff) |
| 3.2 | 新增 admin-bff upstream | 🟡 中 | 添加 3020 upstream，路由 `/admin/*` |
| 3.3 | 添加速率限制 | 🟡 中 | 接入 gateway 层的限流策略 |
| 3.4 | 添加健康检查聚合 | 🟡 中 | `/healthz` 检查所有 upstream 健康 |
| 3.5 | 添加测试 | 🟢 低 | 基本路由测试 |

---

## 4. 执行顺序与依赖

```
Phase 0: 修复现有问题
├── 0.1 修复 web-bff tenant/init (无依赖)
├── 0.2 补全 admin-bff (依赖 0.1 — dashboard 需要 service)
├── 0.3 补全 services policies/ (无依赖)
└── 风险: 🟡 中（涉及服务层修改）

Phase 1: 能力迁移验证
├── 依赖: Phase 0 完成
├── 1.1 验证 web-bff 覆盖 (无依赖)
├── 1.2 更新基础设施配置 (依赖 1.1)
├── 1.3 更新文档 (依赖 1.2)
└── 风险: 🟢 低（纯验证+配置更新）

Phase 2: 删除 servers/api
├── 依赖: Phase 1 完成
├── 风险: 🟡 中（删除后需确认无引用残留）

Phase 3: 完善 Gateway
├── 依赖: Phase 2 完成（upstream 端口确定）
└── 风险: 🟢 低（增量改进）
```

---

## 5. 风险矩阵

| 操作 | 风险 | 回滚难度 | 缓解措施 |
|------|------|---------|---------|
| Phase 0.1 修改 tenant handler | 🟡 中 | 低 | 先写测试确认行为 |
| Phase 0.2 补全 admin-bff | 🟡 中 | 低 | 纯新增代码 |
| Phase 0.3 创建 policies/ | 🟢 低 | 极低 | 空目录+模块声明 |
| Phase 1 验证 | 🟢 无 | 无 | 纯检查 |
| Phase 2 删除 api/ | 🟡 中 | 中 | 先确认无外部引用 |
| Phase 3 Gateway 改进 | 🟢 低 | 低 | 增量修改 |

---

## 6. 验证标准

完成后：

| 验证项 | 预期结果 |
|--------|---------|
| `servers/api/` 不存在 | 已删除 |
| `servers/bff/web-bff/` 覆盖所有 api 端点 | 端点列表等价 |
| `servers/bff/admin-bff/` 功能完整 | 无占位端点，auth 已接线 |
| `servers/gateway/` 正确路由 | 所有 upstream 健康 |
| `just dev-web-bff` | 启动 web-bff 端口 3010 |
| `just dev-admin-bff` | 启动 admin-bff 端口 3020 |
| Gateway 端口 3000 | 正确代理到 web-bff/admin-bff |
| 无编译错误 | `cargo check --workspace` 通过 |
| 无 tests 失败 | `cargo test --workspace` 通过 |

---

## 7. 后续工作（本计划范围外）

1. **Service 结构规范化** — 为所有服务补齐 `src/policies/`、`src/events/`、`src/contracts/`
2. **Event-bus 重新定位** — 它不是 domain service，应移入 `packages/messaging/` 或 `packages/runtime/`
3. **Service 中 `infrastructure/`、`interfaces/`、`sync/` 目录规范化** — 这些不在 codemap.yml 标准结构中，需要明确定位
4. **Admin-bff 租户管理端点** — 实现完整的 CRUD 管理界面
5. **Gateway 生产级配置** — 限流、熔断、重试、服务发现

---

## 8. 总结

**核心发现**: `servers/api` 是一个**已被 web-bff 取代的历史实现**。web-bff 已经是更规范的版本（service 通过 AppState 注入）。重构的主要工作是：

1. **补全 admin-bff**（当前最弱的一环）
2. **修复 web-bff 中的 raw SQL 问题**
3. **删除 servers/api**（清理历史遗留）
4. **完善 gateway 路由**（upstream 端口更新）

**建议执行顺序**: Phase 0 → Phase 1 → Phase 2 → Phase 3，每阶段完成后再进入下一阶段。
