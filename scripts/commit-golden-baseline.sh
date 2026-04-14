#!/usr/bin/env bash
# commit-golden-baseline.sh — Generate and commit golden baselines for ALL generated artifacts
#
# Usage: just commit-golden-baseline
#
# This script:
# 1. Generates fresh platform catalog, contract bindings, and SDKs
# 2. Copies all generated artifacts to verification/golden/ subdirectories
# 3. Reports what was committed
#
# Golden baselines created:
#   - verification/golden/generated-platform/   ← platform catalog
#   - verification/golden/contracts/            ← contract type bindings
#   - verification/golden/sdk-typescript/       ← TypeScript SDK
#   - verification/golden/sdk-rust/             ← Rust SDK

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
GOLDEN_ROOT="$PROJECT_ROOT/verification/golden"

echo "=== Committing golden baselines for all generated artifacts ==="
echo ""

# ── Helper: copy directory to golden baseline ──
copy_to_golden() {
    local name="$1"
    local source_dir="$2"
    local golden_dir="$3"

    mkdir -p "$golden_dir"

    # Copy all files except .gitkeep
    if [ "$(find "$source_dir" -type f ! -name '.gitkeep' 2>/dev/null | wc -l | tr -d ' ')" -gt 0 ]; then
        # Remove old golden files first
        find "$golden_dir" -type f ! -name '.gitkeep' -delete 2>/dev/null || true
        # Copy preserving structure
        (cd "$source_dir" && find . -type f ! -name '.gitkeep' -print0 | cpio -pmd0 "$golden_dir" 2>/dev/null) || \
            (cd "$source_dir" && find . -type f ! -name '.gitkeep' -exec cp --parents {} "$golden_dir/" \; 2>/dev/null) || \
            (rsync -a --exclude='.gitkeep' "$source_dir/" "$golden_dir/" 2>/dev/null) || \
            cp -r "$source_dir/"* "$golden_dir"/ 2>/dev/null || true

        echo "  $name: $(find "$golden_dir" -type f ! -name '.gitkeep' 2>/dev/null | wc -l | tr -d ' ') files"
    else
        echo "  $name: SKIP (no files to copy)"
    fi
}

# Step 1: Generate platform catalog
echo "1. Generating platform catalog..."
cargo run -p platform-generator --quiet -- --platform-dir "$PROJECT_ROOT/platform"
echo "  Platform catalog generated"
echo ""

# Step 2: Copy platform catalog to golden
echo "2. Copying platform catalog to golden baseline..."
copy_to_golden \
    "generated-platform" \
    "$PROJECT_ROOT/platform/catalog" \
    "$GOLDEN_ROOT/generated-platform"
echo ""

# Step 3: Generate contract bindings
echo "3. Generating contract bindings..."
just typegen
echo "  Contract bindings generated"
echo ""

# Step 4: Copy contract bindings to golden
echo "4. Copying contract bindings to golden baseline..."
copy_to_golden \
    "contracts" \
    "$PROJECT_ROOT/packages/contracts/generated" \
    "$GOLDEN_ROOT/contracts"
echo ""

# Step 5: TypeScript SDK (if gen-sdk exists)
echo "5. Checking TypeScript SDK..."
if just --list 2>/dev/null | grep -q 'gen-sdk'; then
    echo "  Running gen-sdk..."
    just gen-sdk
    copy_to_golden \
        "sdk-typescript" \
        "$PROJECT_ROOT/packages/sdk/typescript" \
        "$GOLDEN_ROOT/sdk-typescript"
else
    echo "  SKIP: gen-sdk command not yet defined"
    mkdir -p "$GOLDEN_ROOT/sdk-typescript"
fi
echo ""

# Step 6: Rust SDK (if gen-sdk exists)
echo "6. Checking Rust SDK..."
if just --list 2>/dev/null | grep -q 'gen-sdk'; then
    copy_to_golden \
        "sdk-rust" \
        "$PROJECT_ROOT/packages/sdk/rust" \
        "$GOLDEN_ROOT/sdk-rust"
else
    echo "  SKIP: gen-sdk command not yet defined"
    mkdir -p "$GOLDEN_ROOT/sdk-rust"
fi
echo ""

# Summary
echo "============================================================="
echo "  Golden baselines updated:"
for dir in generated-platform contracts sdk-typescript sdk-rust; do
    golden_dir="$GOLDEN_ROOT/$dir"
    if [ -d "$golden_dir" ]; then
        file_count=$(find "$golden_dir" -type f ! -name '.gitkeep' 2>/dev/null | wc -l | tr -d ' ')
        echo "    - $dir/ ($file_count files)"
    fi
done
echo ""
echo "  Next step: git add $GOLDEN_ROOT && git commit -m 'chore: update golden baselines'"
echo "============================================================="
