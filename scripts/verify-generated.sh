#!/usr/bin/env bash
# verify-generated.sh — Compare freshly generated artifacts against golden baselines
#
# Usage: just verify-generated
# Exit code: 0 if no drift in ANY directory, 1 if drift detected
#
# This script:
# 1. For each generated directory, compares against golden baseline
# 2. Reports drift per directory
#
# Directories checked:
#   - packages/contracts/generated/        → verification/golden/contracts/
#   - platform/catalog/                    → verification/golden/generated-platform/
#   - packages/sdk/typescript/             → verification/golden/sdk-typescript/
#   - packages/sdk/rust/                   → verification/golden/sdk-rust/

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
TEMP_ROOT=$(mktemp -d)

trap 'rm -rf "$TEMP_ROOT"' EXIT

OVERALL_DRIFT=0

# ── Helper: compare source directory against golden baseline ──
# Usage: check_directory "name" "source_dir" "golden_dir"
check_directory() {
    local name="$1"
    local source_dir="$2"
    local golden_dir="$3"

    echo ""
    echo "--- Checking: $name ---"

    # Check if source directory exists
    if [ ! -d "$source_dir" ]; then
        echo "  SKIP: Source directory not found: $source_dir"
        return 0
    fi

    # Check if golden baseline exists
    if [ ! -d "$golden_dir" ]; then
        echo "  SKIP: Golden baseline not found: $golden_dir"
        echo "  Run 'just commit-golden-baseline' to create it."
        return 0
    fi

    # Check if golden baseline is empty (only .gitkeep)
    local golden_file_count
    golden_file_count=$(find "$golden_dir" -type f ! -name '.gitkeep' 2>/dev/null | wc -l | tr -d ' ')
    if [ "$golden_file_count" -eq 0 ]; then
        echo "  SKIP: Golden baseline is empty (no files to compare)"
        echo "  Run 'just commit-golden-baseline' after generating artifacts."
        return 0
    fi

    # Check if source directory is empty (only .gitkeep)
    local source_file_count
    source_file_count=$(find "$source_dir" -type f ! -name '.gitkeep' 2>/dev/null | wc -l | tr -d ' ')
    if [ "$source_file_count" -eq 0 ]; then
        echo "  SKIP: Source directory is empty (only .gitkeep)"
        echo "  Run the appropriate generation command first."
        return 0
    fi

    # Compare recursively
    local drift=0

    # Check each golden file exists in source and matches
    while IFS= read -r -d '' golden_file; do
        local rel_path="${golden_file#$golden_dir/}"
        local source_file="$source_dir/$rel_path"

        if [ ! -f "$source_file" ]; then
            echo "  DRIFT: Missing generated file: $rel_path"
            drift=1
            continue
        fi

        if ! diff -q "$golden_file" "$source_file" > /dev/null 2>&1; then
            echo "  DRIFT: $rel_path differs from baseline"
            drift=1
        else
            echo "  OK: $rel_path"
        fi
    done < <(find "$golden_dir" -type f ! -name '.gitkeep' -print0 2>/dev/null)

    # Check for extra files in source that aren't in golden
    while IFS= read -r -d '' source_file; do
        local rel_path="${source_file#$source_dir/}"
        local golden_file="$golden_dir/$rel_path"

        if [ ! -f "$golden_file" ]; then
            echo "  DRIFT: Extra generated file (not in baseline): $rel_path"
            drift=1
        fi
    done < <(find "$source_dir" -type f ! -name '.gitkeep' -print0 2>/dev/null)

    if [ $drift -eq 1 ]; then
        OVERALL_DRIFT=1
    fi
}

# ── Helper: regenerate and compare (for generators that support custom output dir) ──
# Usage: check_directory_with_regen "name" "source_dir" "golden_dir" "regen_command"
check_directory_with_regen() {
    local name="$1"
    local source_dir="$2"
    local golden_dir="$3"
    local regen_cmd="$4"
    local temp_dir="$TEMP_ROOT/$name"

    echo ""
    echo "--- Checking: $name (with regeneration) ---"

    # Check if golden baseline exists
    if [ ! -d "$golden_dir" ]; then
        echo "  SKIP: Golden baseline not found: $golden_dir"
        echo "  Run 'just commit-golden-baseline' to create it."
        return 0
    fi

    # Check if golden baseline is empty
    local golden_file_count
    golden_file_count=$(find "$golden_dir" -type f ! -name '.gitkeep' 2>/dev/null | wc -l | tr -d ' ')
    if [ "$golden_file_count" -eq 0 ]; then
        echo "  SKIP: Golden baseline is empty (no files to compare)"
        return 0
    fi

    # Regenerate to temp directory
    echo "  Regenerating $name..."
    mkdir -p "$temp_dir"

    # Execute regeneration command
    if ! eval "$regen_cmd" > /dev/null 2>&1; then
        echo "  ERROR: Regeneration failed for $name"
        echo "  Skipping drift check (cannot compare without fresh generation)"
        return 0
    fi

    # Compare recursively
    local drift=0

    while IFS= read -r -d '' golden_file; do
        local rel_path="${golden_file#$golden_dir/}"
        local temp_file="$temp_dir/$rel_path"

        if [ ! -f "$temp_file" ]; then
            echo "  DRIFT: Missing generated file: $rel_path"
            drift=1
            continue
        fi

        if ! diff -q "$golden_file" "$temp_file" > /dev/null 2>&1; then
            echo "  DRIFT: $rel_path differs from baseline"
            drift=1
        else
            echo "  OK: $rel_path"
        fi
    done < <(find "$golden_dir" -type f ! -name '.gitkeep' -print0 2>/dev/null)

    # Check for extra files
    while IFS= read -r -d '' temp_file; do
        local rel_path="${temp_file#$temp_dir/}"
        local golden_file="$golden_dir/$rel_path"

        if [ ! -f "$golden_file" ]; then
            echo "  DRIFT: Extra generated file (not in baseline): $rel_path"
            drift=1
        fi
    done < <(find "$temp_dir" -type f -print0 2>/dev/null)

    if [ $drift -eq 1 ]; then
        OVERALL_DRIFT=1
    fi
}

echo "=== Verifying generated artifacts ==="

# 1. Platform catalog (supports --output-dir, so regenerate to temp)
check_directory_with_regen \
    "platform" \
    "$PROJECT_ROOT/platform/catalog" \
    "$PROJECT_ROOT/verification/golden/generated-platform" \
    "cargo run -p platform-generator --quiet -- --platform-dir '$PROJECT_ROOT/platform' --output-dir '$TEMP_ROOT/platform'"

# 2. Contract bindings (typegen generates in-place, compare source directly)
check_directory \
    "contracts" \
    "$PROJECT_ROOT/packages/contracts/generated" \
    "$PROJECT_ROOT/verification/golden/contracts"

# 3. TypeScript SDK (compare source directly if files exist)
check_directory \
    "sdk-typescript" \
    "$PROJECT_ROOT/packages/sdk/typescript" \
    "$PROJECT_ROOT/verification/golden/sdk-typescript"

# 4. Rust SDK (compare source directly if files exist)
check_directory \
    "sdk-rust" \
    "$PROJECT_ROOT/packages/sdk/rust" \
    "$PROJECT_ROOT/verification/golden/sdk-rust"

# ── Summary ──
echo ""
echo "============================================================="
if [ $OVERALL_DRIFT -eq 0 ]; then
    echo "  No drift detected — all generated artifacts match golden baselines"
    echo "============================================================="
    exit 0
else
    echo "  DRIFT DETECTED — some generated artifacts differ from baselines"
    echo ""
    echo "  To regenerate and update golden baselines, run:"
    echo "    just commit-golden-baseline"
    echo "============================================================="
    exit 1
fi
