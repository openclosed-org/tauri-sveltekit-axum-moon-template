# tauri-sveltekit-axum-moon-template

一个面向桌面优先应用的开源模板，核心是 Rust monorepo，前端使用 SvelteKit。
An open source starter for desktop-first apps with a Rust monorepo core and a SvelteKit UI shell.

它刻意保持业务无关，目标是快速起步，而不是预置产品逻辑。
It is intentionally business-agnostic and optimized for fast bootstrap, not preloaded product logic.

## 简介 | About

这个模板提供跨平台桌面产品的干净基线：
This template gives you a clean baseline for cross-platform desktop products:

- Tauri v2 desktop shell
- SvelteKit 5 frontend (SSR disabled for desktop-oriented runtime flow)
- Rust workspace with layered crate boundaries
- moon task orchestration
- Bun-based frontend toolchain

仓库定位、设计原则、可直接复制的仓库简介文案见 `ABOUT.md`。
For positioning, design principles, and copy-ready repository description, see `ABOUT.md`.

## 你将获得 | What You Get

- 开箱即用的 monorepo 结构（`apps/*` + `crates/*`）与 Cargo + moon 约定。
- `apps/desktop-ui` 下可运行的桌面 UI 脚手架和 Tauri 构建链路。
- Rust 分层骨架：`domain`、`application`、`shared_contracts`、`runtime_server`、`runtime_tauri`。
- `.github/workflows/ci.yml` 中已配置 Rust 与前端质量门禁。
- 社区协作基础文件（`CONTRIBUTING.md`、`CODE_OF_CONDUCT.md`、`SECURITY.md`、`CHANGELOG.md`）。

- Ready-to-run monorepo layout (`apps/*` + `crates/*`) with Cargo + moon conventions.
- Desktop UI app scaffold in `apps/desktop-ui` with Tauri build wiring.
- Rust crate skeletons for `domain`, `application`, `shared_contracts`, `runtime_server`, and `runtime_tauri`.
- CI quality gates for Rust and frontend checks in `.github/workflows/ci.yml`.
- Community health files (`CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`, `SECURITY.md`, `CHANGELOG.md`).

## 当前范围与非目标 | Current Scope and Non-Goals

当前已包含 | Included now:

- Baseline architecture and tooling.
- A minimal Tauri + SvelteKit app entry point.
- Placeholder crates and module boundaries.

默认不包含 | Not included by default:

- Production auth, database, billing, telemetry, or cloud deployment.
- Finished Axum runtime implementation (the runtime crate is scaffolded, not fully wired).
- Product-specific UI, business rules, and domain models.

## 环境要求 | Prerequisites

- Rust `stable`（含 `rustfmt`、`clippy`）
- Node.js `24.0.0`
- Bun `1.3.11`
- 平台对应的 Tauri 系统依赖

- Rust `stable` (with `rustfmt`, `clippy`)
- Node.js `24.0.0`
- Bun `1.3.11`
- System dependencies required by Tauri (platform-specific)

## 快速开始 | Quick Start

1. 在 GitHub 点击 `Use this template` 创建新仓库。
2. 克隆你的新仓库。
3. 安装前端依赖：

1. Click `Use this template` on GitHub and create your repository.
2. Clone your new repository.
3. Install frontend dependencies:

```bash
bun install --cwd apps/desktop-ui
```

4. 执行基础检查：
4. Run baseline checks:

```bash
cargo check --workspace --exclude desktop-ui-tauri
bun run --cwd apps/desktop-ui check
```

5. 启动前端开发：
5. Start frontend development:

```bash
bun run --cwd apps/desktop-ui dev
```

## 模板初始化清单 | Template Bootstrap Checklist

首次发布前请至少更新：
Before first release, update:

- `apps/desktop-ui/src-tauri/tauri.conf.json`
  - `productName`
  - `identifier`
  - window `title`
- `apps/desktop-ui/src/app.html` title
- `apps/desktop-ui/src/routes/+page.svelte` placeholder content
- app icons in `apps/desktop-ui/src-tauri/icons/`
- `LICENSE`, project metadata, and repository About panel

详细检查项见 `docs/BOOTSTRAP_CHECKLIST.md`。
Detailed checklist: `docs/BOOTSTRAP_CHECKLIST.md`.

## 常用命令 | Common Commands

在仓库根目录执行：
From repository root:

```bash
# Rust quality gates
cargo check --workspace --exclude desktop-ui-tauri
cargo test --workspace --exclude desktop-ui-tauri
cargo clippy --workspace --exclude desktop-ui-tauri -- -D warnings
cargo fmt --all -- --check

# Frontend quality gates
bun run --cwd apps/desktop-ui check
bun run --cwd apps/desktop-ui lint
bun run --cwd apps/desktop-ui build

# Frontend dev server
bun run --cwd apps/desktop-ui dev
```

如果你使用 moon：
If you use moon:

```bash
moon run :check
moon run :lint
moon run :test
moon run desktop-ui:dev
```

## 仓库结构 | Repository Layout

```text
.
|- apps/
|  |- desktop-ui/
|     |- src/                 # SvelteKit UI
|     |- src-tauri/           # Tauri app shell
|- crates/
|  |- domain/                 # Domain model and rules boundary
|  |- application/            # Use case orchestration boundary
|  |- shared_contracts/       # Shared DTO/schema boundary
|  |- runtime_server/         # Server runtime boundary (Axum-oriented scaffold)
|  |- runtime_tauri/          # Tauri runtime integration boundary
|- .moon/                     # moon workspace/toolchain config
|- .github/workflows/         # CI pipelines
|- docs/                      # Project-level docs
```

架构说明见 `docs/ARCHITECTURE.md`。
Architecture notes: `docs/ARCHITECTURE.md`.

## 文档索引 | Documentation Index

- `ABOUT.md` - project positioning and copy-ready About text
- `docs/ARCHITECTURE.md` - boundary-driven structure and extension strategy
- `docs/BOOTSTRAP_CHECKLIST.md` - first-day customization checklist
- `CONTRIBUTING.md` - contribution workflow
- `SECURITY.md` - vulnerability reporting process
- `CHANGELOG.md` - release history

## 持续集成 | CI

GitHub Actions 工作流（`.github/workflows/ci.yml`）执行：
GitHub Actions workflow (`.github/workflows/ci.yml`) runs:

- Rust check, fmt check, clippy, test
- Frontend install, check, lint, build

## 安全提示 | Security Note

`apps/desktop-ui/src-tauri/tauri.conf.json` 当前使用 `"csp": null` 以降低模板上手门槛。
`apps/desktop-ui/src-tauri/tauri.conf.json` currently uses `"csp": null` for easier bootstrap.

正式发布前请配置并验证严格的 CSP。
Before production release, define and validate a strict CSP.

## 许可证 | License

本仓库使用 WTF-0 Public License，详见 `LICENSE`。
This repository uses the WTF-0 Public License. See `LICENSE`.
