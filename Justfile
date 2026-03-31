# Justfile - 跨平台开发任务
# 支持: macOS, Linux, Windows

# 设置shell为bash (跨平台)
set shell := ["bash", "-cu"]

default:
	@just --list

# ============================================
# 开发环境
# ============================================

dev:
	moon run app:dev &
	moon run server-api:dev

dev-web:
	moon run app:dev

dev-api:
	moon run server-api:dev

dev-tauri:
	cd apps/client/native/src-tauri && cargo tauri dev

dev-all:
	moon run server-api:dev &
	moon run app:dev &
	cd apps/client/native/src-tauri && cargo tauri dev

# ============================================
# 构建
# ============================================

build-tauri:
	cd apps/client/native/src-tauri && cargo tauri build

build-web:
	moon run app:build

build-api:
	moon run server-api:build

# ============================================
# 测试
# ============================================

test-rust:
	cargo test --workspace

test-web:
	cd apps/client/web/app && bun run test:unit

test-e2e:
	cd apps/client/web/app && bun run test:e2e

# ============================================
# 检查
# ============================================

check:
	moon lint
	moon run :check
	moon test

# ============================================
# 清理
# ============================================

clean:
	cargo clean
