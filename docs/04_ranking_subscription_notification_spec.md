# 04. 排行榜、订阅、资源雷达与通知规范

状态：Normative  
优先级：P0  
目标：定义系统如何把原始 GitHub 数据变成高信噪比信号。

---

## 1. 规范目标

本文档解决四个 P0 问题：

1. 什么样的变化会进入订阅系统；
2. TopK 如何定义和计算；
3. Resource Radar 如何定义、筛选、排序；
4. 什么情况下允许通知用户。

---

## 2. 订阅规范

### 2.1 订阅对象

一个 Subscription 只绑定一个 `Repository`。

v1 不支持：

- 订阅一个 org 作为主对象；
- 订阅 topic 作为主对象；
- 订阅 issue / PR / discussion 作为主对象。

### 2.2 默认订阅模式

v1 默认模式：`TRACKING_STANDARD`

它默认启用：

- `RELEASE_PUBLISHED`
- `TAG_PUBLISHED`
- `DEFAULT_BRANCH_ACTIVITY_DIGEST`

它默认关闭：

- `PR_MERGED_DIGEST`
- 任何 issue/discussion 明细事件

### 2.3 可用更新判定规则

#### Rule U1：Release

若仓库在上次同步之后出现新的已发布 release：

- MUST 生成一个 `RELEASE_PUBLISHED` signal；
- priority 默认 `HIGH`；
- 若 release 是 prerelease，则 MAY 生成 `RELEASE_PRERELEASED`，默认 priority 为 `MEDIUM`。

依据：Releases API 可直接获取发布版信息。[R7]

#### Rule U2：Tag

若仓库在上次同步之后出现新 tag，且该 tag 不属于已知 release：

- MUST 生成一个 `TAG_PUBLISHED` signal；
- priority 默认 `MEDIUM`；
- UI 文案 MUST 说明这是 tag 变化，而不是 release。

#### Rule U3：Default Branch Activity Digest

若仓库默认分支 `pushed_at` 在时间窗内变化，且未产生更高优先级 release/tag signal：

- MAY 生成一个 `DEFAULT_BRANCH_ACTIVITY_DIGEST`；
- 一个 digest 时间窗内最多一个；
- 默认 priority 为 `MEDIUM`；
- 默认不触发即时桌面通知，只进入 digest 和 Home。

解释：

- 这条规则的目的不是“列出每个 commit”，而是告诉用户“这个仓库在最近确实有有效活动”。
- 若后续需要 commit 级摘要，可在 detail 中展开，但不是首页/通知主语义。

#### Rule U4：PR Merged Digest（高级模式）

仅当用户显式开启高级模式：

- 在时间窗内聚合被 merge 的 PR，生成一条 `PR_MERGED_DIGEST`；
- 默认 priority `LOW` 或 `MEDIUM`；
- 不允许逐条 merge 即时推送。

### 2.4 订阅时间窗

合法值：

- `12h`
- `24h`

默认：`24h`

裁决：

- 12h / 24h 本质是产品节奏参数，而不是性能参数；
- 默认 24h 更符合低打扰；
- 12h 适合高关注技术栈或高活跃仓库跟踪者。

### 2.5 订阅同步算法（概念）

```text
for each active subscription:
  fetch repository summary
  fetch latest releases
  fetch latest tags
  compare against local cursors/snapshots
  build normalized signals
  dedupe within window
  persist signals
after repo sync batch:
  generate digest window outputs
```

约束：

- MUST 幂等；
- MUST 允许单 repo 失败而不拖垮整个 batch；
- MUST 记录上次成功同步点；
- SHOULD 使用 ETag 或条件请求优化。

---

## 3. TopK 榜单规范

### 3.1 TopK 的产品定义

TopK 不是对 GitHub 官方 Trending 页面的模仿。

TopK 是由以下要素构成的产品对象：

- `candidate_query`
- `filters`
- `ranking_mode`
- `time_window`
- `k`
- `snapshot_series`

### 3.2 候选集生成

v1 候选集主要来自 GitHub Search Repositories。

#### [核验] Search repositories 能按 `stars`, `forks`, `help-wanted-issues`, `updated` 排序

官方文档列出了 repository search 可用排序参数及 query qualifiers。[R3]

候选集规则：

1. 必须允许按 `language` 过滤；
2. 应允许按 `topic`, `created`, `pushed`, `stars` 阈值过滤；
3. 默认排除 `archived=true`；
4. 默认排除 fork 仓库，除非用户显式开启；
5. 默认对极低热度 repo 施加最小阈值，避免噪音。

### 3.3 v1 支持的 RankingMode

#### Mode T1：`STARS_DESC`

用途：看长期受欢迎项目。  
适用：冷启动、广义发现。

#### Mode T2：`UPDATED_DESC`

用途：看最近活跃更新项目。  
适用：技术雷达、近期观察。

#### Mode T3：`MOMENTUM_24H`

用途：看 24h 内增量显著的项目。  
适用：趋势发现。  
前提：本地已有连续快照。

#### Mode T4：`MOMENTUM_7D`

用途：看 7d 内更稳健的增长。  
适用：避免短期噪声。

### 3.4 TopK 评分规则

#### T1 / T2

- `STARS_DESC` 直接按 `stargazers_count desc`
- `UPDATED_DESC` 直接按 `updated_at desc`

#### T3 / T4

采用产品定义公式：

```text
momentum_score =
  0.50 * norm(star_delta_window)
+ 0.20 * norm(fork_delta_window)
+ 0.30 * norm(updated_recency)
```

说明：

- `star_delta_window` / `fork_delta_window` 来自连续 snapshot 差分；
- `updated_recency` 来自当前时间与 `updated_at` 的距离；
- 公式 MUST 固定版本号，避免 silent change；
- 若快照不足，MUST 降级到 `UPDATED_DESC`。

### 3.5 TopK 快照规范

- 每个保存的 RankingView MUST 有独立快照序列；
- 快照最小粒度与用户刷新频率无关，由 scheduler 决定；
- v1 默认快照周期：`12h`；
- 同步时可以使用缓存结果，但 UI 必须区分“当前视图”与“历史快照对比”。

### 3.6 TopK 变化信号

若某个保存的 RankingView 相比上一快照发生显著变化，可生成 `TOPK_VIEW_CHANGED`。

显著变化建议条件：

- Top 10 中新增项数量 >= 2；或
- Top 10 第一名发生变化；或
- 某订阅中的 repo 新进入 Top 10。

默认规则：

- 只进入 Home / digest；
- 默认不做即时桌面推送。

---

## 4. Resource Radar 规范

### 4.1 设计定位

Resource Radar 不是“AI 工具导航站”。

它的职责是：

- 围绕用户关心的语言/框架/已订阅仓库，发现与 coding agent 生产力相关的 GitHub 资源；
- 以榜单或专题推荐的方式呈现；
- 服务于开发决策，而不是内容消费。

### 4.2 v1 资源来源

v1 资源主源限定为：

- GitHub 仓库搜索结果；
- 与 repo metadata 绑定的显式 topics / tags / language 信息；
- 少量由系统维护的 query templates / curation hints。

v1 不支持：

- 任意站点爬虫；
- 文章聚合；
- 视频内容聚合；
- 用户自由上传资源源。

### 4.3 Resource 分类

v1 只允许以下 `ResourceKind`：

- `MCP_SERVER`
- `SKILL_PACK`
- `AGENT_FRAMEWORK`
- `TEMPLATE`
- `TOOLING`
- `EXAMPLE_REPO`

### 4.4 Resource 归类规则

归类顺序：

1. 显式 curated tag
2. repo topics / name / description 命中规则
3. query template 来源上下文
4. 无法确定时落入 `TOOLING` 或不收录

规则：

- v1 MUST 支持“无法可靠归类则不推荐”；
- 不允许为了凑量把低置信对象强行塞进榜单。

### 4.5 Resource 相关性评分

建议公式：

```text
resource_score =
  0.40 * norm(stack_relevance)
+ 0.25 * norm(star_delta_window)
+ 0.20 * norm(recency)
+ 0.15 * curation_bonus
```

定义：

- `stack_relevance`：与用户显式语言/框架标签的重叠得分
- `star_delta_window`：资源仓库在时间窗内的增长幅度
- `recency`：最近更新时间/最近发布的衰减函数
- `curation_bonus`：系统维护的人工或半人工可信度加分

### 4.6 Resource 新信号

当以下条件成立时，可生成 `RESOURCE_EMERGED`：

- 在用户关注的语言/框架下，出现新的高相关资源；
- resource_score 超过阈值；
- 且该资源在最近 N 天内未向该用户展示过。

默认：

- 进入 Resources 页与 Home；
- 不做即时桌面提醒。

---

## 5. 通知规范

### 5.1 通知层级

1. **即时桌面通知**
2. **Digest**
3. **Home 排序提升**
4. **仅列表出现**

### 5.2 默认通知矩阵

| SignalType | 默认优先级 | 即时桌面通知 | Digest | Home |
|---|---|---:|---:|---:|
| RELEASE_PUBLISHED | HIGH | 是 | 是 | 是 |
| RELEASE_PRERELEASED | MEDIUM | 否 | 是 | 是 |
| TAG_PUBLISHED | MEDIUM | 否 | 是 | 是 |
| DEFAULT_BRANCH_ACTIVITY_DIGEST | MEDIUM | 否 | 是 | 是 |
| PR_MERGED_DIGEST | LOW | 否 | 可选 | 可选 |
| TOPK_VIEW_CHANGED | LOW | 否 | 是 | 是 |
| RESOURCE_EMERGED | LOW | 否 | 是 | 是 |
| RESOURCE_RERANKED | LOW | 否 | 否 | 是 |

### 5.3 通知去噪原则

- 一个 digest 时间窗内，同一 repo 默认最多出现 1 条主信号；
- 若有 release，则 branch digest 不再单独顶层提醒；
- TopK 变化与资源变化默认只能作为次级信号进入 digest，不可抢占 release 级入口。

### 5.4 安静时段

v1 SHOULD 支持安静时段配置。

规则：

- 在安静时段内，不发送即时桌面通知；
- digest 仍可在下个非安静时段展示；
- 信号生成与通知投递必须解耦。

---

## 6. 统一优先级排序

Home / digest 内部默认排序建议：

```text
sort_key =
  priority_weight
+ recency_weight
+ source_type_weight
+ user_affinity_weight
```

其中：

- `priority_weight`: HIGH > MEDIUM > LOW
- `source_type_weight`: release > tag > branch_digest > topk_change > resource
- `user_affinity_weight`: 已订阅仓库 > 关注栈相关的榜单 > 广义资源

要求：

- 同一优先级下排序必须稳定；
- 用户一眼能看出什么最值得先处理。

---

## 7. 缓存与暖机规范

### 7.1 TopK 冷启动

若没有历史快照：

- `MOMENTUM_*` 视图 MUST 显示“暖机中”；
- 系统 MUST 提供退化结果（例如 `UPDATED_DESC`）；
- 不得返回空白且无解释的榜单页。

### 7.2 订阅冷启动

对于新订阅 repo：

- 首次同步 SHOULD 建立 baseline；
- 首次同步不得把历史所有 release/tag 全部视为“新信号”；
- 默认只看基线之后的新变化。

---

## 8. 算法变更纪律

任何涉及以下内容的改动都必须版本化：

- RankingMode 的评分公式
- Resource Radar 的归类规则
- 高/中/低优先级映射
- 显著变化阈值
- 默认订阅事件集合

禁止 silent change。

---

## 9. 本文档结论

v1 的业务引擎不是“把 GitHub 数据渲染出来”，而是把三种不同任务统一翻译为同一种产物：**可解释、可去重、可排序、可通知的 Signal**。

只要这条主线不丢，`TopK`、`Subscriptions`、`Resources` 就不会各自演化成独立的半成品。
