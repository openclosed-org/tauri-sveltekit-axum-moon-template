# Quality — fmt, lint, typecheck, verify, contracts

# 格式化代码（Rust + Web）
fmt:
    moon run repo:fmt

# 修复格式
fmt-fix:
    moon run repo:format-fix

# Lint（Rust clippy + Web biome）
lint:
    moon run repo:quality-lint

# 类型检查（Rust check + Web svelte-check）
typecheck:
    moon run repo:typecheck

# 合约对齐检查（typegen + drift detection + server alignment validation）
contracts-check MODE='warn':
    bun run scripts/validate-contracts.ts --mode {{MODE}}

# 生成合约类型绑定
typegen:
    moon run repo:typegen

# 合约漂移检查 — 检查 generated 目录是否与源码一致（不重新生成）
# 用于 CI 和 scoped gates 中检测未提交的类型变更
# 使用 git diff 检测未提交的变更（不包括已 staged 的变更）
drift-check:
    @echo "=== Checking contract drift ==="
    DRIFT=$(git diff --name-only packages/contracts/generated/ 2>/dev/null) && test -z "$DRIFT" && echo "DRIFT CHECK PASSED" || { echo "DRIFT DETECTED: Run 'just typegen' to regenerate"; echo "$DRIFT"; exit 1; }

# SDK 漂移检查 — 检查 packages/sdk/ 是否有未提交的变更
# 用于 CI 中检测未重新生成 SDK 的漂移
sdk-drift-check:
    @echo "=== Checking SDK drift ==="
    DRIFT=$(git diff --name-only packages/sdk/ 2>/dev/null) && test -z "$DRIFT" && echo "SDK DRIFT CHECK PASSED" || { echo "SDK DRIFT DETECTED: Run 'just typegen' to regenerate"; echo "$DRIFT"; exit 1; }

# 架构边界检查（features / contracts 依赖约束）
boundary-check:
    moon run repo:boundary-check

# 完整质量门禁（fmt + lint + check + test）
verify:
    moon run repo:verify

# ── Backend Lane Commands ─────────────────────────────────────

_primary_packages := '-p kernel -p platform -p runtime -p contracts_api -p contracts_events -p contracts_errors -p adapter-telemetry-otel -p adapter-telemetry-tracing -p storage_turso -p user-service -p tenant-service -p counter-service -p event-bus -p web-bff -p data -p observability'
_auth_optional_packages := '-p contracts_auth -p adapter-google-backend -p auth-service -p authz'
_secondary_packages := '-p pingora-gateway -p outbox-relay-worker -p indexer-worker -p projector-worker -p scheduler-worker -p sync-reconciler-worker -p platform-validator -p dependency-validator -p contract-drift-detector -p topology-validator -p security-validator -p observability-validator -p platform-generator'
_alternative_packages := '-p tenant-service -p storage_surrealdb'

# 默认后端主链静态检查（counter reference chain）
verify-backend-primary:
    cargo fmt {{_primary_packages}} -- --check
    cargo clippy {{_primary_packages}} --all-targets -- -D warnings
    cargo check {{_primary_packages}}

# 可选 auth lane 静态检查（不阻塞默认主链）
verify-auth-optional:
    cargo fmt {{_auth_optional_packages}} -- --check
    cargo clippy {{_auth_optional_packages}} --all-targets -- -D warnings
    cargo check {{_auth_optional_packages}}

# 次级 backend / governance 静态检查
verify-backend-secondary:
    cargo fmt {{_secondary_packages}} -- --check
    cargo clippy {{_secondary_packages}} --all-targets -- -D warnings
    cargo check {{_secondary_packages}}

# 备选 lane 静态检查
verify-backend-alternative:
    cargo fmt {{_alternative_packages}} -- --check
    cargo clippy {{_alternative_packages}} --all-targets -- -D warnings
    cargo check {{_alternative_packages}}

# backend experiments: compile drift across all backend tiers
verify-backend-experiments:
    cargo check {{_primary_packages}} {{_auth_optional_packages}} {{_secondary_packages}} {{_alternative_packages}}

# backend experiments stable-only clippy sweep
lint-backend-experiments:
    cargo clippy {{_primary_packages}} {{_auth_optional_packages}} {{_secondary_packages}} {{_alternative_packages}} --all-targets -- -D warnings

# primary backend coverage lane
coverage-backend-primary:
    cargo llvm-cov {{_primary_packages}} --lcov --output-path lcov.info

# 生成目录分类索引（Agent 搜索优先级提示）
gen-dir-cats:
    bun run scripts/gen-directory-categories.ts

# ── 供应链安全 ─────────────────────────────────────────────

# k6 基线压测（默认连接 localhost:3010，可自定义 BASE_URL）
k6-baseline BASE_URL='http://localhost:3010' DURATION='30s' VUS='5':
    k6 run \
      -e BASE_URL={{BASE_URL}} \
      -e TEST_DURATION={{DURATION}} \
      -e TEST_VUS={{VUS}} \
      {{justfile_directory()}}/scripts/k6-baseline.js

# Trivy 漏洞扫描（默认扫描整个项目文件系统，可指定 IMAGE）
scan-vuln IMAGE='' EXIT_CODE='0':
    @if [ -n "{{IMAGE}}" ]; then \
        echo "=== Trivy image scan: {{IMAGE}} ===" ; \
        trivy image --severity HIGH,CRITICAL --exit-code {{EXIT_CODE}} {{IMAGE}} ; \
    else \
        echo "=== Trivy filesystem scan ===" ; \
        trivy fs --severity HIGH,CRITICAL --exit-code {{EXIT_CODE}} {{justfile_directory()}} ; \
    fi

# Syft SBOM 生成（默认生成 JSON 格式，输出到 stdout 或文件）
gen-sbom IMAGE='' FORMAT='json' OUTPUT='-':
    @if [ -n "{{IMAGE}}" ]; then \
        echo "=== Syft SBOM: image {{IMAGE}} ===" ; \
        if [ "{{OUTPUT}}" = "-" ]; then \
            syft {{IMAGE}} -o {{FORMAT}} ; \
        else \
            syft {{IMAGE}} -o {{FORMAT}}={{OUTPUT}} ; \
        fi ; \
    else \
        echo "=== Syft SBOM: workspace ===" ; \
        if [ "{{OUTPUT}}" = "-" ]; then \
            syft dir:{{justfile_directory()}} -o {{FORMAT}} ; \
        else \
            syft dir:{{justfile_directory()}} -o {{FORMAT}}={{OUTPUT}} ; \
        fi ; \
    fi

# Cosign 镜像签名（需要 COSIGN_EXPERIMENTAL=1 时用 keyless 模式）
sign-image IMAGE:
    @echo "=== Cosign sign: {{IMAGE}} ==="
    cosign sign {{IMAGE}}

# Cosign 镜像验证
verify-image IMAGE:
    @echo "=== Cosign verify: {{IMAGE}} ==="
    cosign verify {{IMAGE}}

# 一键跑全量供应链检查（本地 gate）
supply-chain:
    @echo "=== Supply Chain Gate ==="
    just scan-vuln '' 0
    just gen-sbom
    @echo "=== Supply Chain Gate PASSED ==="

# ── 内部工具 ──────────────────────────────────────────────────

_require TOOL CMD:
    @bun -e "const { spawnSync } = require('child_process'); const t = '{{TOOL}}'; const r = spawnSync(process.platform === 'win32' ? 'where' : 'which', [t], { shell: process.platform === 'win32' }); if (r.status !== 0) { console.log('Missing {{TOOL}} — install with: {{CMD}}'); process.exit(1); }"
