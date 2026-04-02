# Phase 8: Desktop Native Features — Research Report

**Date:** 2026-03-29

## 1. System Tray (Tauri 2 Rust API)

**Source:** https://tauri.app/learn/system-tray/ + Medium article by Sjobeiri

### Key Findings

- Rust API: `tauri::tray::TrayIconBuilder`, `tauri::menu::MenuItemBuilder`, `tauri::menu::MenuBuilder`
- `Cargo.toml` 必须给 `tauri` 添加 feature: `["tray-icon"]`
- 示例代码：
  ```rust
  use tauri::menu::MenuItemBuilder;
  use tauri::tray::TrayIconBuilder;

  // 在 setup() 中
  let hide = MenuItemBuilder::new("Hide").id("hide").build(app).unwrap();
  let quit = MenuItemBuilder::new("Quit").id("quit").build(app).unwrap();
  let menu = MenuBuilder::new(app)
      .items(&[&hide, &quit])
      .build()
      .unwrap();

  TrayIconBuilder::new()
      .icon(app.default_window_icon().unwrap().clone())
      .menu(&menu)
      .on_menu_event(|app, event| match event.id().as_ref() {
          "quit" => app.exit(0),
          "hide" => {
              let window = app.get_webview_window("main").unwrap();
              window.hide().unwrap();
          }
          _ => {}
      })
      .build(app)?;
  ```
- 图标复用: `app.default_window_icon()` — 使用应用默认图标
- Show/Hide 可以用一个 toggle item，根据窗口状态动态切换文本

### 注意事项

- Windows 上 emoji 颜色渲染不可靠，菜单文字应保持纯文本
- 需要在 `#[cfg(desktop)]` 条件编译，避免移动端影响
- 功能级别: Windows ✅, Linux ✅, macOS ✅

---

## 2. Window State Plugin

**Source:** https://github.com/tauri-apps/plugins-workspace/tree/v2/plugins/window-state (README.md)

### Key Findings

- **依赖已就绪**: `tauri-plugin-window-state` 已声明在 workspace deps 和 `src-tauri/Cargo.toml`
- 注册: `.plugin(tauri_plugin_window_state::Builder::default().build())`
- 窗口关闭时自动保存状态，下次启动自动恢复
- 手动保存: `app.save_window_state(StateFlags::all())`
- 手动恢复: `window.restore_state(StateFlags::all())`
- 前端 JS: `saveWindowState(StateFlags.ALL)`, `restoreStateCurrent(StateFlags.ALL)`
- 功能级别: Windows ✅, Linux ✅, macOS ✅

### 注意事项

- 窗口最小尺寸需要单独设置: `WindowBuilder::new().min_inner_size(800.0, 600.0)` — 但这是在 `tauri.conf.json` 中配置的
- 可能需要在 `tauri.conf.json` 的 window 配置中添加 `minWidth`/`minHeight`
- Capabilities 需要 window-state 权限

---

## 3. Single Instance Plugin

**Source:** https://v2.tauri.app/plugin/single-instance/

### Key Findings

- **独立插件**: `tauri-plugin-single-instance` — 需要新增依赖
- 需要添加到 `Cargo.toml`: `tauri-plugin-single-instance = "2.4.0"`
- 注册方式:
  ```rust
  #[cfg(desktop)]
  {
      builder = builder.plugin(tauri_plugin_single_instance::init(|app, args, cwd| {
          let _ = app.get_webview_window("main")
                     .expect("no main window")
                     .set_focus();
      }));
  }
  ```
- 回调参数: `AppHandle`, `args: Vec<String>`, `cwd: String`
- 如果窗口隐藏到托盘: 需要在回调中 `window.show()` + `window.set_focus()`
- Linux 使用 DBus, 需要为 Snap/Flatpak 声明权限 (MVP 不需要考虑)
- **不需要 JS 权限配置** — 纯 Rust 侧插件
- 功能级别: Windows ✅, Linux ✅, macOS ✅

### 注意事项

- 需要 `#[cfg(desktop)]` 条件编译
- 如果窗口隐藏到托盘，需要 show + focus:
  ```rust
  let window = app.get_webview_window("main").expect("no main window");
  let _ = window.show();
  let _ = window.set_focus();
  ```

---

## 4. Error Handling (Panic Hook + Event Emission)

**Source:** 基于 Tauri 2 事件系统 + 标准库 panic hook

### Key Findings

- `std::panic::set_hook()` 可以设置全局 panic 钩子
- Tauri 2 事件发射: `app.emit("error-event", payload)` — 所有前端监听器都能收到
- 前端事件监听: `listen('error-event', (event) => { ... })` from `@tauri-apps/api/event`
- Toast 可以用 Svelte 组件或 shadcn-svelte 的 toast 组件

### 实现策略

1. 在 setup() 中保存 `AppHandle` 克隆
2. 设置 panic hook: `std::panic::set_hook(Box::new(move |info| { ... }))`
3. panic hook 中: 提取消息, `app.emit("panic", { message })`
4. 前端: 监听 `panic` 事件, 显示 toast

### 注意事项

- Panic hook 是全局的，会覆盖任何已有 hook
- 需要在 fallback 情况下处理非 String panic payload
- IPC 命令的 `Result<T, String>` 错误已经由 Tauri 自动传到前端 `invoke().catch()`
- 前端可以在全局错误边界中捕获 invoke 错误并显示 toast

---

## 5. 窗口最小尺寸

### Key Findings

- Tauri 2: 在 `tauri.conf.json` 的 window 配置中添加 `minWidth`/`minHeight`:
  ```json
  {
    "app": {
      "windows": [
        {
          "width": 1200,
          "height": 800,
          "minWidth": 800,
          "minHeight": 600,
          "resizable": true
        }
      ]
    }
  }
  ```
- 或者通过 Rust API: `WindowBuilder::new().min_inner_size(800.0, 600.0)`

---

## Summary: New Dependencies Required

| 依赖 | 类型 | 是否已存在 |
|------|------|-----------|
| `tauri-plugin-window-state` | Cargo.toml | ✅ 已存在 |
| `tauri-plugin-single-instance` | Cargo.toml | ❌ 需新增 |
| `tauri` feature `tray-icon` | Cargo.toml | ❌ 需添加 |
| `@tauri-apps/plugin-window-state` | package.json | ❓ 需检查 |

## Summary: Code Changes Required

| 文件 | 改动 |
|------|------|
| `Cargo.toml` (root) | 添加 single-instance 到 workspace deps |
| `src-tauri/Cargo.toml` | 添加 single-instance dep, 给 tauri 添加 tray-icon feature |
| `src-tauri/src/lib.rs` | 注册 window-state + single-instance 插件, setup tray, setup panic hook |
| `src-tauri/capabilities/default.json` | 添加 tray 权限 |
| `tauri.conf.json` | 添加 minWidth/minHeight |
| 前端 toast 组件 | 新增全局错误 toast 组件 |
