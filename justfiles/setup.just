# Setup — toolchain installation and verification

# 安装/校验所有工具版本（Rust、Node、Bun）
setup:
    mise install

# 安装本地 NATS Server（用于本地开发和测试）
setup-nats:
    bash {{justfile_directory()}}/tools/nats/setup.sh

# 安装 bun 依赖
setup-deps:
    bun install

# 一键初始化（工具 + 依赖 + 诊断）
bootstrap:
    mise install
    bun install
    bun run scripts/doctor.ts

# 检查工具链状态（不安装，只验证）
doctor:
    @echo "=== Toolchain ==="
    @mise current rust  || echo "MISSING: rust — run: just setup"
    @mise current node  || echo "MISSING: node — run: just setup"
    @mise current bun   || echo "MISSING: bun  — run: just setup"
    @echo ""
    @echo "=== Versions ==="
    @cargo --version 2>/dev/null || echo "cargo not in PATH"
    @node --version   2>/dev/null || echo "node not in PATH"
    @bun --version    2>/dev/null || echo "bun not in PATH"

# 项目级健康检查（工具 + 配置文件）
doctor-full:
    bun run scripts/doctor.ts

# 安装 sccache 编译缓存（加速重复编译 80%+）
setup-sccache:
    @echo "=== Installing sccache ==="
    @bun -e "const { spawnSync } = require('child_process'); const r = spawnSync(process.platform === 'win32' ? 'where' : 'which', ['sccache'], { shell: process.platform === 'win32' }); if (r.status !== 0) { console.log('Installing sccache...'); spawnSync('cargo', ['install', 'sccache', '--locked'], { stdio: 'inherit', shell: process.platform === 'win32' }); } else { console.log('sccache already installed'); }"
    @echo ""
    @echo "=== sccache 安装完成 ==="
    @echo ""
    @echo "请配置环境变量（二选一）："
    @echo ""
    @echo "方案 A - 项目级 .env（推荐）："
    @echo "  1. 复制 .env.example 到 .env"
    @echo "  2. 取消注释 sccache 相关行"
    @echo ""
    @echo "方案 B - mise 环境变量："
    @echo "  mise set SCCACHE_DIR={{justfile_directory()}}/.sccache"
    @echo "  mise set RUSTC_WRAPPER=sccache"
    @echo ""
    @echo "配置完成后运行：just setup-sccache-verify"

# 验证 sccache 是否生效
setup-sccache-verify:
    @echo "=== sccache 验证 ==="
    @echo ""
    @echo "--- sccache 状态 ---"
    @bun -e "const { spawnSync } = require('child_process'); const r = spawnSync('sccache', ['--show-stats'], { stdio: 'inherit', shell: process.platform === 'win32' });"
    @echo ""
    @echo "--- 环境变量检查 ---"
    @bun -e "const wrapper = process.env.RUSTC_WRAPPER; console.log(wrapper === 'sccache' ? '✓ RUSTC_WRAPPER=sccache' : wrapper ? '⚠ RUSTC_WRAPPER=' + wrapper + ' (应该是 sccache)' : '✗ RUSTC_WRAPPER 未设置');"
    @echo ""
    @echo "--- .cargo/config.toml 检查 ---"
    @bun -e "const fs = require('fs'); const c = fs.readFileSync('.cargo/config.toml', 'utf8'); console.log(c.includes('rustc-wrapper') ? '✓ rustc-wrapper 已配置' : '✗ rustc-wrapper 未设置');"

# 安装 cargo-hakari 统一依赖解析（减少重复编译 30%+）
setup-hakari:
    @bun -e "const { spawnSync } = require('child_process'); const r = spawnSync(process.platform === 'win32' ? 'where' : 'which', ['cargo-hakari'], { shell: process.platform === 'win32' }); if (r.status !== 0) { console.log('Installing cargo-hakari...'); spawnSync('cargo', ['install', 'cargo-hakari'], { stdio: 'inherit', shell: process.platform === 'win32' }); } else { console.log('cargo-hakari already installed: ' + spawnSync('cargo', ['hakari', '--version'], { shell: process.platform === 'win32' }).stdout?.toString().trim()); }"
    @echo "Generating unified dependency resolution..."
    cargo hakari generate
    cargo hakari manage-deps --yes
    @echo "hakari configured — run cargo build to see speedup"

# 验证 hakari 是否生效
setup-hakari-verify:
    @echo "=== hakari status ==="
    @bun -e "const fs = require('fs'); const exists = fs.existsSync('.config/hakari.toml'); console.log(exists ? '✓ .config/hakari.toml exists' : '✗ .config/hakari.toml missing');"
    @bun -e "const fs = require('fs'); const c = fs.readFileSync('Cargo.toml', 'utf8'); console.log(c.includes('workspace-hack') ? '✓ workspace-hack in members' : '✗ workspace-hack NOT in members');"

# 安装覆盖率工具（一次性）
setup-coverage:
    @bun -e "const { spawnSync } = require('child_process'); ['cargo-llvm-cov', 'cargo-sweep'].forEach(t => { const r = spawnSync(process.platform === 'win32' ? 'where' : 'which', [t], { shell: process.platform === 'win32' }); if (r.status !== 0) spawnSync('cargo', ['install', t.replace('cargo-', '')], { stdio: 'inherit', shell: process.platform === 'win32' }); }); spawnSync('rustup', ['component', 'add', 'llvm-tools'], { stdio: 'inherit', shell: process.platform === 'win32' }); console.log('Coverage tools ready');"

# 检查工具链版本
toolchain-check:
    rustc --version && bun --version && moon --version && echo "ALL OK"
