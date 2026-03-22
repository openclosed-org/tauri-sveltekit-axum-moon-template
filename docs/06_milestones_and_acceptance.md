# 06. 开发里程碑与验收规范

状态：Normative  
优先级：P0 / P1  
目标：把文档母稿转为可执行的工程交付节奏。

---

## 1. 里程碑设计原则

1. 每个里程碑必须产出用户可感知价值；
2. 每个里程碑必须有明确退出条件；
3. 每个里程碑结束后都应能冻结一层语义，而不是继续漂移；
4. 验收必须以场景和结果为准，而不是“代码写完了”。

---

## 2. 里程碑总览

| 里程碑 | 目标 | 产出关键词 |
|---|---|---|
| M0 | 规格冻结 | 术语、边界、默认值、对象模型冻结 |
| M1 | 基础壳与本地状态 | Tauri shell、SQLite、基础 UI、设置、token 管理 |
| M2 | 订阅主链路 | repo 搜索/添加订阅、同步、signal、digest、通知 |
| M3 | TopK 榜单引擎 | saved views、快照、排名、变化信号 |
| M4 | Resource Radar | 资源分类、相关性、资源榜 |
| M5 | 硬化与发布 | 性能、安全、测试、发布包、回归稳定 |

---

## 3. M0：规格冻结

### 3.1 目标

完成所有 P0 语义冻结，保证团队用同一种语言工作。

### 3.2 必交付物

- 本文档集完整初稿
- `domain-vocabulary.yaml`
- `v1-defaults.yaml`
- 关键对象与状态机评审记录

### 3.3 退出条件

1. `Usable Update` 定义冻结；
2. `TopK` 定义冻结；
3. 一级导航冻结；
4. 默认通知矩阵冻结；
5. 数据主模型冻结。

### 3.4 验收标准

- 任意一个新加入的工程师，在无口头说明下可读懂：
  - 产品做什么；
  - 什么不做；
  - 什么叫 signal；
  - 什么叫 topk；
  - 什么叫 resource。

---

## 4. M1：基础壳与本地状态

### 4.1 目标

形成可运行的桌面应用骨架。

### 4.2 范围

- Tauri + SvelteKit 基础工程
- SQLite 初始化与 migration
- 本地设置页
- GitHub token 输入与安全存储
- Home / TopK / Subscriptions / Resources / Rules 基础路由壳
- 基础 repo 搜索与结果列表（不含完整 TopK）

### 4.3 必交付物

- 可打包的桌面应用
- 本地数据库可创建并迁移
- token 可写入安全存储并读取
- 应用离线启动不崩溃

### 4.4 退出条件

1. 应用可以冷启动；
2. 路由和布局稳定；
3. 基础 contract 打通；
4. 错误状态可显示；
5. 配置持久化可工作。

### 4.5 场景验收

#### AC-M1-01
给定首次打开应用的用户，应用应能完成：

- 进入主界面；
- 填写 GitHub token；
- 保存成功；
- 再次重启仍能读到配置状态（不要求直接展示 token 明文）。

#### AC-M1-02
当设备离线时：

- 应用仍可启动；
- 页面不白屏；
- 明确提示“当前为离线/缓存模式”。

---

## 5. M2：订阅主链路

### 5.1 目标

完成从“发现 repo”到“获得可用更新 signal”的最小闭环。

### 5.2 范围

- 搜索 repo 并创建 subscription
- 同步 release / tag / default branch digest
- signal 去重、持久化、已读状态
- digest 生成
- 桌面通知（仅 high）

### 5.3 必交付物

- Subscription CRUD
- Sync scheduler v1
- Signal list / detail
- Digest 视图
- Notification adapter

### 5.4 退出条件

1. 同一个 release 不会重复生成 signal；
2. 一个 repo 在 24h digest 内最多一个 branch digest signal；
3. 高优先级 release 能被桌面提醒；
4. 用户可标记 signal 为已读/已处理；
5. Home 能展示“自上次访问以来”的变化。

### 5.5 场景验收

#### AC-M2-01：新增订阅
用户从 repo 搜索结果中点击“订阅”，系统应：

- 创建 subscription；
- 使用默认模式；
- 在列表中可见；
- 在下一次同步中参与检查。

#### AC-M2-02：新 release
若某订阅 repo 在基线之后发布新的 release，系统应：

- 生成 1 条且仅 1 条 `RELEASE_PUBLISHED` signal；
- priority 为 `HIGH`；
- signal 可打开原 release 页；
- 若开启默认通知，则应收到桌面提醒。

#### AC-M2-03：默认分支活动摘要
若某 repo 在 24h 内有活动但无 release/tag，系统应：

- 生成 0 或 1 条 `DEFAULT_BRANCH_ACTIVITY_DIGEST`；
- 该 signal 不应即时骚扰用户；
- 应出现在 digest 与 Home 中。

#### AC-M2-04：幂等
同一同步任务重复执行两次，不应新增重复 signal。

---

## 6. M3：TopK 榜单引擎

### 6.1 目标

完成趋势发现主链路。

### 6.2 范围

- RankingView CRUD
- Search candidate generation
- RankingSnapshot 存储
- `STARS_DESC`, `UPDATED_DESC`, `MOMENTUM_*`
- TopK 变化 signal

### 6.3 必交付物

- 可保存多个榜单视图
- 可查看当前结果与快照时间
- 冷启动时 momentum 视图能优雅降级
- 可从榜单项一键订阅 repo

### 6.4 退出条件

1. 一个 RankingView 可稳定复算；
2. 视图快照可回看；
3. 排行模式切换不会破坏视图定义；
4. TopK 变化信号可生成且不过度打扰。

### 6.5 场景验收

#### AC-M3-01
用户新建“Rust Recent Movers”视图后，系统应：

- 保存 filters 和 ranking mode；
- 生成快照；
- 列出前 K 项；
- 显示 snapshot 时间；
- 支持从任意 item 一键订阅。

#### AC-M3-02
若当前没有历史快照，进入 `MOMENTUM_7D` 视图时：

- 系统必须提示“暖机中”；
- 返回合理的降级排序结果；
- 不得空白或报错。

---

## 7. M4：Resource Radar

### 7.1 目标

完成与 code agent 提效相关的资源发现能力。

### 7.2 范围

- Resource ingestion v1
- Resource kind 分类
- stack relevance 评分
- Resource 专题榜
- why recommended 展示

### 7.3 必交付物

- 至少支持若干语言/框架组合的资源榜
- ResourceCard detail
- Resource 与 repo / subscription 的关联跳转

### 7.4 退出条件

1. 资源卡片有明确 kind；
2. 资源榜能按语言/框架过滤；
3. 每条推荐都能解释理由；
4. 误分类率与噪音可接受。

### 7.5 场景验收

#### AC-M4-01
给定用户显式关注 `Rust + Axum`，系统应能在资源页给出：

- 相关资源列表；
- 每条资源的种类；
- 推荐原因；
- 源仓库入口。

#### AC-M4-02
若资源无法可靠归类：

- 系统应降级为不推荐或落入低优先级通用工具；
- 不得伪装成高相关 MCP / Skill 资源。

---

## 8. M5：硬化与发布

### 8.1 目标

把原型提升为可以稳定交付的 v1。

### 8.2 范围

- 性能优化
- 速率预算与 backoff
- 错误处理完善
- 打包与签名
- 崩溃恢复
- 回归测试
- 可观测性与诊断信息

### 8.3 必交付物

- 发布包
- 回归测试报告
- 性能预算达标结果
- 安全清单
- Known Issues 列表

### 8.4 退出条件

1. 主要路径可重复稳定演示；
2. 无 P0 数据丢失 / 重复通知 / 崩溃缺陷；
3. 关键性能目标达标或有明确降级解释；
4. 文档与实现一致。

---

## 9. Definition of Done（全局）

一个功能只有同时满足以下条件才算完成：

1. 语义与规范一致；
2. 代码、测试、文档同步完成；
3. 能用场景复现；
4. 错误路径已验证；
5. 不引入已知 P0 语义漂移。

---

## 10. 发布阻塞项（Release Blockers）

以下任一项存在，则禁止发布：

1. 同一 release 重复通知；
2. signal 无法打开原始依据；
3. momentum 榜单在无快照时直接报错；
4. token 明文落盘；
5. 应用离线启动失败；
6. Home 无法区分 stale / fresh 数据；
7. 任何页面将 issue/discussion 噪音误当成默认可用更新。

---

## 11. 本文档结论

正确的里程碑顺序不是“把所有页面都做出来”，而是：

- 先把语义冻结；
- 再打通订阅闭环；
- 再做榜单与资源；
- 最后做硬化。

否则看起来功能很多，但用户并没有得到一个可信的技术雷达。
