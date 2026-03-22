# 08. 外部核验资料与裁决说明

状态：Reference  
说明：本文件只记录本次母稿写作时已核验的外部约束。凡未在此处核验的外部事实，文档中应视为设计建议或显式假设。

---

## R1. Tauri Architecture

来源：Tauri 官方文档 `Tauri Architecture`  
已核验事实：

- Tauri 用 Rust 工具和渲染于 WebView 的 HTML 组合构建桌面应用；
- WebView 与 Rust backend 通过消息传递交互；
- Tauri 依赖 OS WebView，因此体积较小。

用途：

- 支撑桌面工作台、本地优先、Rust 核心逻辑的架构裁决。

核验状态：`[核验]`

---

## R2. Tauri + SvelteKit 集成指南

来源：Tauri 官方文档 `SvelteKit | Tauri`  
已核验事实：

- 推荐使用 `@sveltejs/adapter-static`；
- Tauri 不支持 server-based frontend solution；
- 若使用 prerender，构建阶段的 `load` 不可访问 Tauri API；
- 官方更推荐 SPA 模式。

用途：

- 支撑“UI 用 SvelteKit，但业务核心不能依赖 SvelteKit server runtime”的架构结论。

核验状态：`[核验]`

---

## R3. GitHub REST Search API

来源：GitHub 官方文档 `REST API endpoints for search` / `Search repositories`  
已核验事实：

- Search API 每次搜索最多返回 `1000` 结果；
- 搜索会在最多 `4000` 个匹配仓库范围内执行；
- 认证搜索默认 `30 req/min`，未认证默认 `10 req/min`；
- repository search 可按 `stars`, `forks`, `help-wanted-issues`, `updated` 排序；
- query 长度与逻辑操作符数量有上限；
- search 可能因超时返回 `incomplete_results=true`。

用途：

- 支撑 TopK 必须以缓存视图和快照驱动，而不是把 Search API 当作实时无限接口。

核验状态：`[核验]`

---

## R4. GitHub REST Rate Limits

来源：GitHub 官方文档 `Rate limits for the REST API` / `Rate limit status`  
已核验事实：

- 未认证请求默认 `60 req/hour`；
- 认证用户请求默认 `5000 req/hour`；
- 存在 secondary rate limits；
- rate limit status endpoint 可查询当前额度状态。

用途：

- 支撑 token 建议、预算分池和 scheduler 设计。

核验状态：`[核验]`

---

## R5. GitHub Events API

来源：GitHub 官方文档 `REST API endpoints for events`  
已核验事实：

- Events API 不是为 real-time 用例设计；
- event latency 可能在 `30s` 到 `6h`；
- 推荐使用 `ETag`；
- 返回 `X-Poll-Interval` 指示建议轮询间隔。

用途：

- 支撑“不能承诺实时订阅”的产品裁决；
- 支撑轮询与缓存优化策略。

核验状态：`[核验]`

---

## R6. GitHub Repository Webhooks

来源：GitHub 官方文档 `Types of webhooks` / `Creating webhooks` / `REST API endpoints for repository webhooks`  
已核验事实：

- 只有 repository owner 或 admin access 才能创建和管理 repository webhooks；
- 创建 webhook 需要 write 级 webhooks 权限。

用途：

- 支撑“无法对任意公开仓库使用通用 webhook 方案”的架构裁决。

核验状态：`[核验]`

---

## R7. GitHub Releases API

来源：GitHub 官方文档 `REST API endpoints for releases` / `Managing releases in a repository`  
已核验事实：

- Releases API 可列出仓库的 releases；
- 发布的 release 对公开资源可见；
- releases 与普通 git tags 不完全等价，未关联 release 的 tag 需通过 tags 端点查看。

用途：

- 支撑 `RELEASE_PUBLISHED` 和 `TAG_PUBLISHED` 的语义拆分。

核验状态：`[核验]`

---

## R8. Turso Embedded Replicas

来源：Turso 官方文档 `Embedded Replicas`  
已核验事实：

- Embedded Replicas 支持 `read-your-writes` 语义；
- 其他副本通过 `sync()` 或周期同步看到变更；
- 适合本地 + 云同步场景。

用途：

- 支撑“云增强模式下优先考虑 Turso”的演进裁决。

核验状态：`[核验]`

---

## R9. GitHub Repositories API（Topics / Tags）

来源：GitHub 官方文档 `REST API endpoints for repositories`  
已核验事实：

- 可获取 repository topics；
- 可获取 repository tags；
- 公开仓库在无额外权限下可访问基础 metadata 端点。

用途：

- 支撑 Resource Radar 的标签与 tag/release 语义实现。

核验状态：`[核验]`

---

## 裁决说明

### C1. 为什么 TopK 不是“GitHub Trending API”

本次核验资料中确认了 GitHub 官方 Search API 与其排序能力，但未核验到一个官方公开的、可直接等价为“Trending API”的 REST endpoint。

因此本项目对 TopK 的处理方式是：

- **不假装存在一个权威 Trending 接口；**
- 直接把 TopK 定义为产品级排行榜语义；
- 用 Search + Snapshot + Ranking Formula 实现。

标记：`[推断]`，但该推断已足够支持 v1 产品定义。

### C2. 为什么不把 Web 作为 v1 正式主端

该结论主要来自产品/架构推理，而非某单一官方文档直接规定：

- 桌面工作台更契合本地缓存、通知、安全存储与高密度工作流；
- v1 若强行同时正式交付多端，会显著提升复杂度与同步压力。

标记：`[设计裁决]`。
