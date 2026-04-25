# Build — workspace build, single-service build, cross-compile

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

# Build Tauri desktop app
build-tauri:
    cargo tauri build --manifest-path apps/desktop/src-tauri/Cargo.toml

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
