# Phase 03: Application Pages - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-03-28
**Phase:** 03-application-pages
**Areas discussed:** Page routing & layout structure, Admin sidebar navigation, Responsive adaptation strategy, Page-specific content & behavior

---

## Page routing & layout structure

| Option | Description | Selected |
|--------|-------------|----------|
| Flat routes (/login, /counter, /admin) | Each page is a top-level route directory. Simple, flat URL structure. | |
| Grouped routes (/auth/login, /app/counter, /app/admin) | SvelteKit route groups: (auth) for login, (app) for counter+admin with sidebar. Different layouts per group. | ✓ |
| Nested routes (/counter, /admin, /admin/*) | Admin has its own +layout.svelte with sidebar. Login and counter are flat. | |

**User's choice:** Grouped routes (/auth/login, /app/counter, /app/admin)

**Follow-up — Layout sharing:**

| Option | Description | Selected |
|--------|-------------|----------|
| Shared (app) layout | Both counter and admin pages share the same sidebar layout under (app)/ | ✓ |
| Separate layouts | Counter has its own minimal layout (no sidebar). Admin has sidebar layout. | |

**User's choice:** Shared (app) layout

---

## Admin sidebar navigation

**User requested:** 作为一个 mobile first 的跨端应用的现代布局设计

**Agent proposed:** Bottom Tab Bar（移动）+ Collapsible Sidebar（桌面）

| Option | Description | Selected |
|--------|-------------|----------|
| 同意这个方案 | 移动底部 tab bar + 桌面可折叠 sidebar，modern 跨端模式 | ✓ |
| 都用 sidebar + 抽屉 | 全部用 sidebar，移动端 sidebar 变成抽屉式（hamburger menu） | |
| 都用 bottom tab bar | 全部用 bottom tab bar，桌面端也用底部导航 | |

**User's choice:** 同意这个方案

---

## Responsive adaptation strategy

**Login page:**

| Option | Description | Selected |
|--------|-------------|----------|
| 居中卡片（全尺寸） | 移动端和桌面端都是居中的卡片式表单，只是卡片宽度/间距自适应 | |
| 移动端全屏 + 桌面居中卡片 | 移动端全屏无边距，桌面端居中卡片 | ✓ |
| 上下/左右分栏 | 移动端品牌区域在上方表单在下方，桌面端左右分栏 | |

**User's choice:** 移动端全屏 + 桌面居中卡片

**Counter page:**

| Option | Description | Selected |
|--------|-------------|----------|
| 始终居中 + 移动端加大按钮 | 数字和按钮始终居中，移动端按钮更大更容易点击（h-12+） | ✓ |
| 移动端垂直 + 桌面水平 | 移动端垂直排列，桌面端水平排列 | |

**User's choice:** 始终居中 + 移动端加大按钮

**Admin page:**

| Option | Description | Selected |
|--------|-------------|----------|
| 单列堆叠 → 多列网格 | 移动端单列卡片堆叠，桌面端2-3列网格 | |
| 固定2列全尺寸 | 固定 2 列，移动端卡片缩小 | |
| 移动端列表 + 桌面卡片网格 | 移动端列表式，桌面端卡片网格 | ✓ |

**User's choice:** 移动端列表 + 桌面卡片网格

---

## Page-specific content & behavior

**Login page:**

| Option | Description | Selected |
|--------|-------------|----------|
| Logo + Google 按钮 + 简洁标语 | App logo + name, Google sign-in Button, "欢迎使用" subtitle | |
| Logo + Google 按钮 + email 占位 | Logo, Google 按钮, 额外有 email 输入框（placeholder 禁用态）暗示未来支持 | ✓ |

**User's choice:** Logo + Google 按钮 + email 占位

**Counter page:**

| Option | Description | Selected |
|--------|-------------|----------|
| 数字 + +/- + reset（可负数） | 大数字显示，+/- 按钮，reset 按钮，可以到负数。展示 $state rune | ✓ |
| 数字 + +/- + step 选项（最小0） | 大数字显示，+/- 按钮，最小值为0。加入 step 选项（+1, +5, +10） | |

**User's choice:** 数字 + +/- + reset（可负数）

**Admin dashboard:**

| Option | Description | Selected |
|--------|-------------|----------|
| 4张 placeholder 统计卡片 | 3-4 Card 组件显示 placeholder 统计数据 | |
| 标题 + 描述 + 1个 Card | 最简 placeholder，不需要假数据 | |
| 图表 placeholder + 统计卡片 | 侧边栏导航项 + 主区域显示图表 placeholder + 统计卡片 | ✓ |

**User's choice:** 图表 placeholder + 统计卡片

---

## Agent's Discretion

- Exact number of statistics cards and chart types in admin placeholder
- Tab bar icon choices (Lucide icons from @lucide/svelte)
- Sidebar transition animation (slide, fade, or instant)
- Route transition animations between pages

## Deferred Ideas

- Google OAuth actual implementation — Phase 6
- Admin dashboard real data — future phase
- Counter persistence (localStorage / backend sync) — future phase
- Page transition animations — can add later, not blocking
- Settings page with full preferences — expand when needed
