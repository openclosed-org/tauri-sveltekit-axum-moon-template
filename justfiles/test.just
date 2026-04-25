# Testing — all test and coverage commands

# ── Rust 测试 ───────────────────────────────────────────────

# 日常开发：快速单元测试（无覆盖率）
test:
    moon run repo:test-unit

# 使用 nextest 运行测试（推荐）
test-nextest:
    cargo nextest run --workspace

# cargo-hack feature powerset 检查
test-hack:
    cargo hack check --workspace --feature-powerset

# cargo-mutants 变异测试
test-mutants:
    cargo mutants --workspace

# 集成测试（被 --ignore 标记的）
test-integration:
    cargo test --workspace -- --ignored

# 所有 Rust 测试套件（nextest + coverage + hack）
test-all-rust:
    moon run repo:test-all-rust

# ── 覆盖率 ──────────────────────────────────────────────────

# 本地覆盖率：生成 LCOV（供 CI/Codecov 用）
test-coverage:
    just _require cargo-llvm-cov "cargo install cargo-llvm-cov"
    moon run repo:test-coverage

# 本地覆盖率：生成 HTML 并在浏览器打开
test-coverage-html:
    just _require cargo-llvm-cov "cargo install cargo-llvm-cov"
    moon run repo:test-coverage-html

# 清理覆盖率产物
test-coverage-clean:
    just _require cargo-llvm-cov "cargo install cargo-llvm-cov"
    moon run repo:test-coverage-clean

# ── E2E 测试 ────────────────────────────────────────────────

# Web Playwright E2E
test-e2e:
    moon run repo:test-e2e

# 运行仓库全量 E2E 门禁（runtime preflight + Web + Tauri）
test-e2e-full:
    moon run repo:test-e2e-full

# 运行 Tauri 桌面 E2E（tauri-playwright）
test-desktop:
    moon run repo:test-desktop

# 快速运行 E2E（使用 e2e profile,跳过优化编译）
test-desktop-fast:
    cargo build --manifest-path apps/desktop/src-tauri/Cargo.toml --profile e2e --features e2e-testing --target-dir target
    bun --cwd {{justfile_directory()}}/apps/desktop/tests/e2e run test:desktop:core

# ── 聚合 ────────────────────────────────────────────────────

# 所有前端测试（check + lint + unit + e2e + desktop）
test-all-frontend:
    moon run repo:test-all-frontend

# ── Newman API E2E 测试 ────────────────────────────────────

# Newman API E2E 测试（自动启动服务器 → 运行测试 → 清理）
test-api-e2e:
    bun {{justfile_directory()}}/servers/bff/web-bff/tests/e2e/run-web-e2e.ts

# Newman API E2E 测试（仅运行，假设服务器已在运行）
test-api-e2e-only:
    bunx newman run {{justfile_directory()}}/servers/bff/web-bff/tests/e2e/collection.json \
      -e {{justfile_directory()}}/servers/bff/web-bff/tests/e2e/env.json \
      --bail \
      --reporter-cli \
      --reporter-junit-extra \
      --reporter-junit-extra-output {{justfile_directory()}}/servers/bff/web-bff/tests/e2e/report.xml

# 默认后端主链验证：counter-service + tenant-service + web-bff
test-backend-primary:
    cargo test -p kernel -p platform -p runtime -p contracts_api -p contracts_events -p contracts_errors -p adapter-telemetry-otel -p adapter-telemetry-tracing -p storage_turso -p user-service -p tenant-service -p counter-service -p event-bus -p web-bff -p data -p observability -- --test-threads=4

# 可选 auth lane 验证：auth-service + authz + auth contracts/adapters
test-auth-optional:
    cargo test -p contracts_auth -p adapter-google-backend -p auth-service -p authz

# 次级 backend / governance 测试
test-backend-secondary:
    cargo test -p pingora-gateway -p outbox-relay-worker -p indexer-worker -p projector-worker -p scheduler-worker -p sync-reconciler-worker -p platform-validator -p dependency-validator -p contract-drift-detector -p topology-validator -p security-validator -p observability-validator -p platform-generator -- --test-threads=4

# 备选 lane 测试
test-backend-alternative:
    cargo test -p tenant-service -p storage_surrealdb -- --test-threads=4
