# E2E: Demo Counter

> Playwright E2E 测试：计数器完整链路。

## 测试场景

1. 用户打开 counter 页面
2. 点击 "Increment" 按钮
3. 计数值从 0 → 1
4. 刷新页面后值保持（持久化验证）
5. 点击 "Reset" 按钮
6. 计数值回到 0

## 实现状态

⚠️ 待实现。当前仅有 README 说明。

## 技术选型

- 框架：Playwright (已有 playwright.config.ts)
- 运行：`just e2e` 或 `npx playwright test`
- 目标 URL：`http://localhost:5173/app/counter`
