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

# 生成目录分类索引（Agent 搜索优先级提示）
gen-dir-cats:
    bun run scripts/gen-directory-categories.ts

# ── 内部工具 ──────────────────────────────────────────────────

_require TOOL CMD:
    @bun -e "const { spawnSync } = require('child_process'); const t = '{{TOOL}}'; const r = spawnSync(process.platform === 'win32' ? 'where' : 'which', [t], { shell: process.platform === 'win32' }); if (r.status !== 0) { console.log('Missing {{TOOL}} — install with: {{CMD}}'); process.exit(1); }"
