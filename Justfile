# Justfile — 跨平台开发入口
# 复杂编排由 moon 负责，Just 只暴露稳定入口
set shell := ["bash", "-cu"]

default:
	@just --list

# ── 核心入口 ────────────────────────────────────────────────

# 安装依赖 + 工具链检查
setup:
	moon run repo:setup

# 启动全栈开发（web + api + desktop）
dev:
	moon run repo:dev-fullstack

# 质量验证（fmt + lint + typecheck + test）
verify:
	moon run repo:verify

# 运行单元测试
test:
	moon run repo:test-unit

# 运行类型生成
typegen:
	moon run repo:typegen

# ── 扩展入口 ────────────────────────────────────────────────

# 启动 Web 开发
dev-web:
	moon run repo:dev-web

# 启动 API 开发
dev-api:
	moon run repo:dev-api

# 启动桌面应用开发
dev-desktop:
	moon run repo:dev-desktop

# 运行 E2E 测试
test-e2e:
	moon run repo:test-e2e

# 运行 Lint
lint:
	moon run repo:lint-repo

# 格式化代码
fmt:
	moon run repo:fmt

# 工具链状态检查
doctor:
	moon run repo:doctor

# 清理构建产物
clean:
	cargo clean

# ── 发布（Phase 9 实现）─────────────────────────────────────

# 发布 Desktop
release:
	moon run repo:release-dry-run

# 运行评估
evals:
	moon run repo:evals-run
