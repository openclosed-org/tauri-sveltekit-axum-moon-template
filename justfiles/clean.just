# Clean — cleanup commands for caches and build artifacts

# 暴力清理所有 cargo 缓存（慢，下次全量重建）
clean:
    cargo clean

# 智能清理 7 天未访问的构建缓存
clean-sweep:
    just _require cargo-sweep "cargo install cargo-sweep"
    cargo sweep --time 7

# 清理不在当前 Cargo.toml 中的依赖缓存
clean-sweep-deps:
    just _require cargo-sweep "cargo install cargo-sweep"
    cargo sweep --installed

# 仅清理覆盖率产物
clean-coverage:
    just _require cargo-llvm-cov "cargo install cargo-llvm-cov"
    cargo llvm-cov clean

# 清理测试运行产物（.tmp 目录）
clean-tmp:
    @echo "清理 .tmp 测试产物..."
    rm -rf .tmp/phase3-smoke .tmp/run
    @echo "✓ .tmp 测试产物已清理"

# 截断 .tmp 中的大日志文件（超过 10MB 的截断到保留最后 5 万行）
clean-tmp-truncate:
    @echo "截断大日志文件（>10MB，保留最后 5 万行）..."
    @find .tmp -name "*.log" -size +10M 2>/dev/null | while read f; do \
        echo "  截断 $$f"; \
        tail -n 50000 "$$f" > "$$f.tmp" && mv "$$f.tmp" "$$f"; \
    done
    @echo "✓ 大日志文件已截断"

# 全量清理：构建缓存 + 测试产物
clean-all: clean-tmp clean
    @echo "✓ 全量清理完成"

# 查看 sccache 缓存状态（当前不可用）
sccache-stats:
    @echo "sccache currently disabled — see .env for details"

# 清理 sccache 缓存（当前不可用）
sccache-clean:
    @echo "sccache currently disabled — see .env for details"
