# 02-03 Summary

## 完成内容
- 创建 `+layout.svelte`：导入 app.css，Svelte 5 $props() + {@render} children
- 创建 9 个 UI 组件包装器：

| 组件 | 说明 | 基础库 |
|------|------|--------|
| Button | 4 variant × 3 size | bits-ui Button.Root |
| Input | styled input, bind:value | 原生 input |
| Card | content container | 原生 div |
| Badge | 5 variant status badge | 原生 span |
| Switch | toggle switch | bits-ui Switch |
| Dialog | compound modal | bits-ui Dialog |
| Select | single select | bits-ui Select |
| Dropdown | dropdown menu | bits-ui DropdownMenu |
| Toast | notification alert | 原生 div |

- 创建 barrel export (`$lib/components/index.ts`)

## 组件约定
- 所有组件接受 `class` prop，通过 `cn()` 合并
- 所有组件使用 `dark:` 前缀工具类
- compound 组件同时导出 bits-ui 子部件 (DialogParts, SelectParts, DropdownParts)

## 验证
- ✅ svelte-check: 0 errors
- ✅ vite build: 成功 (577ms)

## 注意事项
- `@pqoqubbw/icons` 在 npm 上 404，已从 package.json 移除（需要单独处理）
- `typescript` 版本从固定 5.5.0 改为 ^5.5.0（npm 无法解析固定版本）
- Select 组件仅支持 single select 模式（multiple 模式需直接使用 bits-ui）
