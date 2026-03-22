# 07. 质量、性能、安全与运行规范

状态：Normative  
优先级：P0 / P1  
目标：确保 v1 不是“能跑”，而是“可长期使用”。

---

## 1. 非功能目标

系统必须在以下四类维度可接受：

1. **性能**：启动快、切换快、缓存命中快；
2. **可靠性**：同步可重试、结果可恢复、状态不混乱；
3. **安全性**：token 不泄露，日志不泄密；
4. **可维护性**：问题可观测，模块可替换，语义不漂移。

---

## 2. 性能预算

### 2.1 启动与交互预算

建议目标（参考硬件：近三年中端开发者笔记本）：

- Warm start 到可交互：`< 1.5s`
- Cold start 到主界面可用：`< 3.0s`
- Home / TopK 页面切换：`< 200ms` 本地响应
- 列表首屏渲染：`< 300ms`（使用缓存时）

说明：

- 这些是工程目标，不是绝对物理保证；
- 若超预算，必须给出定位与降级策略。

### 2.2 同步预算

建议目标：

- 50 个活跃订阅的一轮常规同步：在大部分请求命中条件缓存时 `< 30s`
- 10 个保存 RankingView 的一轮刷新：`< 60s`（含 search budget 节流）

必须条件：

- 超时或受限时，系统必须优雅退化，而不是阻塞 UI。

---

## 3. 可靠性策略

### 3.1 幂等

以下操作 MUST 幂等：

- 同步 release/tag
- 生成 signal
- 生成 digest
- 记录 delivery

### 3.2 崩溃恢复

- 同步任务中途崩溃后，下一次启动 MUST 可继续工作；
- 半完成的 snapshot 不得污染已完成快照；
- 临时状态与最终状态必须分离。

### 3.3 部分失败隔离

- 单个 repo 失败不影响其他 repo；
- 单个 RankingView 失败不影响其他视图；
- 单个 Resource scoring 失败不影响主订阅链路。

---

## 4. GitHub API 运行纪律

### 4.1 预算隔离

- Search budget 与 core budget MUST 分池；
- 高优先级订阅同步应优先于资源雷达刷新；
- 用户主动刷新可以暂时提高优先级，但不可绕开总限流。

### 4.2 Backoff 与重试

- 429 / rate-limited：指数退避 + respect reset time
- 5xx：有限次数重试
- 4xx 业务错误：不盲重试，必须分类记录

### 4.3 缓存与条件请求

- 能使用 ETag/If-None-Match 的端点 SHOULD 使用；[R5]
- 对 Search 类端点必须设置本地 TTL；
- 所有重复查询都应优先命中本地快照/缓存。

---

## 5. 安全规范

### 5.1 凭据与隐私

- token MUST 存于安全存储，不得明文写入 SQLite、日志、崩溃报告；
- token 输入框 MUST 支持覆盖与清除；
- 导出诊断日志时 MUST 脱敏；
- 若后续支持私有仓库，相关缓存与云同步策略必须重审。

### 5.2 最小权限

- v1 SHOULD 优先建议用户使用最小可行权限 token；
- 若当前功能仅需公开仓库访问，UI 应明确说明这一点；
- 不得默认申请无关 scope。

### 5.3 本地数据安全

- 所有持久化文件路径应可识别与可清除；
- 需要提供“清除本地缓存与本地配置”的安全入口；
- 清除缓存不应误删安全存储中的 token，除非用户显式选择。

---

## 6. 测试策略

### 6.1 测试金字塔

1. **单元测试**
   - Signal 去重键
   - 排名公式
   - 资源归类规则
   - 状态机迁移

2. **集成测试**
   - SQLite migration
   - GitHub adapter mapping
   - scheduler + persistence
   - digest generation

3. **契约测试**
   - Rust DTO -> 前端消费模型一致性
   - YAML 枚举 -> 代码枚举一致性

4. **端到端测试**
   - 首次启动
   - 新建订阅
   - 同步并生成 release signal
   - 创建 TopK 视图
   - 资源页过滤与推荐原因展示

### 6.2 必测异常路径

- 无网络启动
- Search 429
- release endpoint 403/404
- token 无效
- 缓存损坏
- 数据库 migration 中断
- 重复同步同一窗口

---

## 7. 可观测性规范

### 7.1 必须记录的运行指标

- app startup duration
- active subscriptions count
- ranking view refresh count
- signal generated count by type
- duplicate signal prevented count
- digest generation duration
- GitHub API request count by endpoint class
- 304 hit rate
- rate limit remaining snapshot
- notification delivery success rate

### 7.2 必须记录的错误维度

- endpoint name
- http status class
- retryability
- request budget pool
- affected object id (`repo_id` / `ranking_view_id` / `resource_id`)
- user-visible degradation state

### 7.3 诊断原则

日志必须对工程师有用，但不得泄漏敏感信息。

---

## 8. UI 质量基线

虽然本文件不提供 AI 设计提示词，但必须明确 UI 质量硬约束：

1. 首页能清楚区分“新信号”和“缓存结果”；
2. 所有列表项状态清晰：未读、已读、已处理、过期；
3. 不允许出现无法解释来源的推荐；
4. 任何通知都必须能在应用内定位到对应 signal；
5. 所有关键路径都应支持键盘快速操作。

---

## 9. 发布与回滚

### 9.1 发布前清单

- migration 测试通过
- token 安全测试通过
- 重复通知回归通过
- 无网络模式回归通过
- momentum 暖机降级回归通过
- macOS/Windows 基础打包验证通过

### 9.2 回滚原则

若发布后出现以下问题，必须考虑紧急回滚或 hotfix：

- token 泄漏风险
- 大规模重复通知
- 应用无法启动
- 本地数据损坏
- Home 页面核心信号丢失

---

## 10. 运行模式下的用户体验纪律

- 同步中：可以浏览缓存，不阻塞 UI；
- 数据 stale：要提示，但不惊慌式报错；
- 受限流影响：明确告诉用户是延迟刷新，不是数据消失；
- 没有数据：给出“为何没有”的解释，而不是空白页。

---

## 11. 本文档结论

真正能让 `geek taste` 被长期使用的，不是某一个炫功能，而是：

- 它打开得快；
- 它不吵；
- 它不乱；
- 它出问题时能解释；
- 它让用户相信这里看到的信号是可信的。
