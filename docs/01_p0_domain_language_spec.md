# 01. P0 领域语言与语义规范

状态：Normative  
优先级：P0  
适用对象：所有研发、测试、产品、设计与文档编写者

---

## 1. 目的

本文档用于消除以下类型的歧义：

1. 同一术语被不同模块赋予不同含义；
2. 同一事件在 UI、数据库、通知中表现不一致；
3. “排行榜 / 订阅 / 资源”三套能力各自发明自己的对象和状态；
4. 后续实现阶段出现“这个词到底是什么意思”的返工。

本文档中的定义具备**规范性约束**。

---

## 2. 规范性关键字

- **MUST**：强制，违反即为实现错误或设计错误。
- **SHOULD**：强建议，若不遵循，必须记录理由与替代方案。
- **MAY**：可选，不影响规范一致性。
- **MUST NOT**：禁止。

---

## 3. 核心对象定义

### 3.1 Repository

`Repository` 指一个 GitHub 仓库对象，是 `owner/name` 唯一命名空间下的源对象。

Repository MUST 至少包含以下属性：

- `repo_id`：GitHub 仓库唯一 ID
- `full_name`：`owner/name`
- `owner`
- `name`
- `html_url`
- `default_branch`
- `primary_language`
- `topics[]`
- `stargazers_count`
- `forks_count`
- `archived`
- `disabled`
- `updated_at`
- `pushed_at`

去歧义：

- `updated_at` 不是“有可用更新”的同义词。
- `pushed_at` 不是“应该立即通知”的同义词。
- `Repository` 不是 `Signal`。

### 3.2 Subscription

`Subscription` 指用户与 Repository 之间的**跟踪关系**，不是 GitHub 原生 watch/star 关系。

Subscription MUST 包含：

- 订阅对象：一个 `Repository`
- 事件范围：允许哪些 `SignalType`
- 频率策略：`DIGEST_12H` / `DIGEST_24H` / `IMMEDIATE_HIGH_ONLY`
- 优先级策略：哪些信号可触发桌面通知
- 生命周期状态：`ACTIVE` / `PAUSED` / `ARCHIVED`

去歧义：

- “订阅”是本产品内的概念，不等于 GitHub 的 `watch`。
- “取消订阅”不应删除历史信号，应改变订阅状态。

### 3.3 Signal

`Signal` 指一个**经过归一化、可展示、可去重、可通知、可追溯**的变化对象。

Signal MUST 满足：

1. 有明确来源（source）；
2. 有明确所属对象（通常是一个 Repository 或 Resource）；
3. 有明确类型（`SignalType`）；
4. 有明确时间戳；
5. 有可稳定复算的 dedupe key；
6. 有可选的优先级（priority）；
7. 能被用户标记为已读 / 已处理；
8. 能打开原始依据。

去歧义：

- 原始 GitHub API payload 不是 Signal。
- UI 上的一行列表项是 Signal 的表现，不是 Signal 本身。

### 3.4 RankingView

`RankingView` 指一个**可重复生成**的榜单视图定义，由以下四类输入决定：

1. 候选查询（query template）
2. 过滤条件（filters）
3. 排序模式（ranking mode）
4. K 值（top size）

去歧义：

- RankingView 是“榜单定义”，不是“榜单结果”。
- RankingSnapshot 才是某时刻的榜单结果。

### 3.5 Resource

`Resource` 指与 code agent 生产力相关的 GitHub 资源对象。

v1 中，Resource 主要来源于 GitHub 仓库或由 GitHub 仓库承载的项目。

Resource MUST 至少有：

- `resource_kind`
- `source_repo_id`
- `tags[]`
- `languages[]`
- `framework_tags[]`
- `agent_tags[]`
- `score_inputs`
- `source_url`

去歧义：

- Resource 不是任意网页条目。
- v1 中 Resource 默认以 GitHub 为主源，而不是泛互联网抓取对象。

---

## 4. 关键术语定义

### 4.1 TopK

`TopK` 指在一个 RankingView 上，应用过滤与排序后返回的前 K 个结果。

规范：

- K MUST 为显式参数，不得隐式写死在 UI 文案里。
- v1 允许的 K 值集合：`10, 20, 50, 100`。
- 若候选不足 K，则返回实际数量。

### 4.2 可用更新（Usable Update）

`Usable Update` 指**足够值得普通跟踪者知晓**的更新，而不是“任何仓库活动”。

v1 中，可用更新 MUST 仅从以下集合中产生：

- `RELEASE_PUBLISHED`
- `TAG_PUBLISHED`
- `DEFAULT_BRANCH_ACTIVITY_DIGEST`
- `PR_MERGED_DIGEST`（仅高级模式，默认关闭）

以下活动 v1 MUST NOT 直接视为可用更新：

- 单个 commit push
- 单个 issue comment
- 单个 discussion comment
- 单个 star/fork 变化

### 4.3 Digest

`Digest` 指在给定时间窗内对多个 Signal 的批量摘要。

Digest MUST：

- 有明确时间窗；
- 有明确来源集合；
- 可追溯到组成它的 Signal 列表；
- 不能替代原始 Signal 存储。

### 4.4 Snapshot

`Snapshot` 指某个时刻记录下来的候选对象状态，用于后续比较与增量计算。

Snapshot 不是事件；Snapshot 是用来计算趋势和变化的材料。

### 4.5 Relevance

`Relevance` 指一个 Resource 或 RankingItem 与用户当前技术栈/兴趣集合的相关程度。

v1 中相关性 MUST 基于显式标签重叠计算，不依赖黑盒推荐模型。

### 4.6 Priority

`Priority` 指 Signal 对通知与首页排序的影响程度。

v1 标准优先级：

- `HIGH`
- `MEDIUM`
- `LOW`

规则：

- `HIGH` 才允许默认桌面提醒。
- `MEDIUM` 默认只进入 digest 与首页。
- `LOW` 默认只进入列表，不主动提醒。

---

## 5. 枚举规范

### 5.1 SignalType

v1 合法枚举：

- `RELEASE_PUBLISHED`
- `RELEASE_PRERELEASED`
- `TAG_PUBLISHED`
- `DEFAULT_BRANCH_ACTIVITY_DIGEST`
- `PR_MERGED_DIGEST`
- `TOPK_VIEW_CHANGED`
- `RESOURCE_EMERGED`
- `RESOURCE_RERANKED`

说明：

- `TOPK_VIEW_CHANGED` 表示某个已保存的榜单视图发生足够显著的结果变化。
- `RESOURCE_EMERGED` 表示某技术栈下出现新的高相关资源。
- `RESOURCE_RERANKED` 表示资源排序显著变化；默认不发桌面通知。

### 5.2 SubscriptionState

- `ACTIVE`
- `PAUSED`
- `ARCHIVED`

语义：

- `ACTIVE`：参与同步与通知。
- `PAUSED`：保留历史，不参与同步和通知。
- `ARCHIVED`：只保留记录，不再显示于默认活跃列表。

### 5.3 SignalState

- `NEW`
- `SEEN`
- `ACKED`
- `ARCHIVED`

语义：

- `NEW`：尚未在 UI 中展示给用户。
- `SEEN`：已被用户浏览，但未明确处理。
- `ACKED`：用户已确认/处理。
- `ARCHIVED`：不再出现在默认工作区。

### 5.4 RankingMode

- `STARS_DESC`
- `UPDATED_DESC`
- `MOMENTUM_24H`
- `MOMENTUM_7D`
- `CURATED_RELEVANCE`

说明：

- `MOMENTUM_*` 需要依赖 Snapshot 计算增量；在冷启动没有历史快照时 MUST 优雅降级。
- `CURATED_RELEVANCE` 仅用于资源榜，不用于通用 repo 榜。

### 5.5 ResourceKind

- `MCP_SERVER`
- `SKILL_PACK`
- `AGENT_FRAMEWORK`
- `TEMPLATE`
- `TOOLING`
- `EXAMPLE_REPO`

说明：

- `SKILL_PACK` 指用于增强 coding agent 的可复用技能包、流程包、任务模板、指令集或其等价形式。
- v1 不对“是否真实符合某协议完整规范”做自动认证，除非存在明确来源证据。

---

## 6. 时间语义规范

### 6.1 时间窗

v1 的 digest 时间窗合法值为：

- `12h`
- `24h`

规则：

- 时间窗 MUST 由用户配置时区解释；
- 若用户未配置，使用设备本地时区；
- 服务端模式下仍以用户时区为准，而不是服务器时区。

### 6.2 自上次访问以来

`Since last visit` 指用户上次成功进入前台并完成主页可交互后的时间点。

该语义 MUST NOT 与“上次同步时间”混淆。

---

## 7. 去重与归因规范

### 7.1 去重原则

同一个外部事实不得在同一逻辑时间窗内生成多个等价 Signal。

示例：

- 同一个 release 不得因多次轮询生成多个 `RELEASE_PUBLISHED`。
- 同一个 24h digest 窗内的默认分支活动，只允许一个 `DEFAULT_BRANCH_ACTIVITY_DIGEST`。

### 7.2 归因原则

每个 Signal MUST 能归因到以下至少一项：

- GitHub release ID
- tag name + repo
- repo full_name + time bucket
- ranking_view_id + snapshot_pair
- resource_id + ranking window

### 7.3 冲突裁决

若同一时间窗内既检测到 `RELEASE_PUBLISHED` 又检测到 `TAG_PUBLISHED`：

- 若 tag 属于该 release，对用户展示 MUST 以 `RELEASE_PUBLISHED` 为主；
- `TAG_PUBLISHED` MAY 作为内部证据，但不应重复提醒。

---

## 8. 排行榜语义规范

### 8.1 榜单不是搜索结果页

TopK MUST 被视为**有状态、可比较、可保存**的产品对象，而不是一次性搜索返回。

### 8.2 榜单必须稳定

同一 `RankingView` 在相同输入、相同时间点、相同缓存状态下 MUST 返回相同结果顺序。

### 8.3 榜单必须可解释

用户必须能理解：

- 它看的是哪个范围；
- 它按什么排序；
- 为什么这条结果在前面；
- 该结果能否一键订阅。

---

## 9. 通知语义规范

### 9.1 通知不是显示层特效

通知必须由 Signal 驱动，而不是由 UI 页面自行判断触发。

### 9.2 通知优先级默认值

- `HIGH`：默认可桌面提醒 + digest 收录
- `MEDIUM`：默认仅 digest + Home
- `LOW`：默认仅列表显示

### 9.3 通知克制原则

v1 MUST NOT：

- 对 TopK 的每次轻微排名变化发即时提醒；
- 对 Resource 的常规重排发即时提醒；
- 对默认分支每个 commit 发提醒。

---

## 10. 错误与退化语义

### 10.1 Stale

`STALE` 指数据可显示，但新鲜度已超过设计阈值。

`STALE` 不是 `ERROR`。

### 10.2 Degraded

`DEGRADED` 指某功能可部分工作，但不能满足完整语义。

示例：

- 无历史 Snapshot 时，`MOMENTUM_24H` 降级为 `UPDATED_DESC`；
- 无网络时，只展示缓存，不执行同步。

### 10.3 Failed

`FAILED` 指一次同步或查询未成功完成，且当前结果不可作为新状态。

FAILED MUST 记录错误原因与可重试性。

---

## 11. 术语禁用清单

以下模糊词 MUST NOT 直接出现在实现说明和需求验收中，除非被更具体定义替代：

- “热门”
- “最新”
- “重大更新”
- “趋势”
- “推荐”
- “智能排序”
- “相关项目”

替代方式：

- 用明确时间窗、排序模式、阈值或评分规则替代。

---

## 12. 规范摘要

本产品的所有讨论，最终都要落回以下最小句法：

- 用户订阅的是 `Repository`；
- 系统产生的是 `Signal`；
- 用户查看的是 `RankingView` / `RankingSnapshot`；
- 系统推荐的是 `Resource`；
- 通知和首页都只是这些对象的视图，而不是额外发明的新概念。
