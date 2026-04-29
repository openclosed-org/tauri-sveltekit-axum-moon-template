# Build control plane
#
# This file owns artifact creation and compile-oriented package operations. It is
# intentionally narrower than verification: build recipes produce binaries or
# perform direct compile actions, while broader quality evidence lives in verify.just.

# Build entire workspace
build:
    moon run repo:build

# Build workspace in release mode
build-release:
    cargo build --workspace --release

# Build a single crate
build-pkg PACKAGE='':
    @if [ -z "{{PACKAGE}}" ]; then \
        echo "Usage: just build-pkg <package-name>"; \
        exit 1; \
    fi
    cargo build -p {{PACKAGE}}

# Cross-compile for Linux musl (static binary)
build-linux-musl:
    cargo build --target x86_64-unknown-linux-musl --release

# Cross-compile for macOS (aarch64)
build-macos-aarch64:
    cargo build --target aarch64-apple-darwin --release

# Build API server only
build-api:
    cargo build -p web-bff --release

# 检查 storage-turso 编译
check-storage-turso:
    cargo check -p web-bff

# Analyze binary size
bloat:
    cargo bloat --release --crates

# Check all crates compile without linking
check:
    moon run repo:check

# Generate cargo-hakari workspace-hack
hakari:
    cargo hakari generate --diff
    cargo hakari manage-deps
