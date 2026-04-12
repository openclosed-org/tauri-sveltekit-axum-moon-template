#!/usr/bin/env bash
# verify-generated.sh — Compare freshly generated platform catalog against golden baseline
#
# Usage: just verify-generated
# Exit code: 0 if no drift, 1 if drift detected
#
# This script:
# 1. Regenerates platform catalog to a temp directory
# 2. Compares against golden baseline (verification/golden/generated-platform/)
# 3. Reports drift if any

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
GOLDEN_DIR="$PROJECT_ROOT/verification/golden/generated-platform"
TEMP_DIR=$(mktemp -d)

trap 'rm -rf "$TEMP_DIR"' EXIT

echo "=== Verifying generated platform artifacts ==="
echo ""

# Check if golden baseline exists
if [ ! -d "$GOLDEN_DIR" ]; then
    echo "WARNING: Golden baseline directory not found: $GOLDEN_DIR"
    echo "Run 'just commit-golden-baseline' first to create the baseline."
    echo ""
    echo "Skipping drift check (no baseline to compare against)."
    exit 0
fi

echo "1. Regenerating platform catalog..."
cargo run -p platform-generator --quiet -- --platform-dir "$PROJECT_ROOT/platform" --output-dir "$TEMP_DIR"

if [ $? -ne 0 ]; then
    echo "ERROR: Platform generation failed!"
    exit 1
fi

echo "2. Comparing against golden baseline..."

# Compare file by file
DRIFT=0
DRIFT_FILES=""

for golden_file in "$GOLDEN_DIR"/*; do
    filename=$(basename "$golden_file")
    temp_file="$TEMP_DIR/$filename"

    if [ ! -f "$temp_file" ]; then
        echo "  DRIFT: Missing generated file: $filename"
        DRIFT=1
        DRIFT_FILES="$DRIFT_FILES $filename"
        continue
    fi

    if ! diff -q "$golden_file" "$temp_file" > /dev/null 2>&1; then
        echo "  DRIFT: $filename differs from baseline"
        DRIFT=1
        DRIFT_FILES="$DRIFT_FILES $filename"
    else
        echo "  OK: $filename"
    fi
done

# Check for extra files in temp that aren't in golden
for temp_file in "$TEMP_DIR"/*; do
    filename=$(basename "$temp_file")
    golden_file="$GOLDEN_DIR/$filename"

    if [ ! -f "$golden_file" ]; then
        echo "  DRIFT: Extra generated file (not in baseline): $filename"
        DRIFT=1
        DRIFT_FILES="$DRIFT_FILES $filename"
    fi
done

echo ""
echo "============================================================="
if [ $DRIFT -eq 0 ]; then
    echo "  No drift detected — generated artifacts match golden baseline"
    echo "============================================================="
    exit 0
else
    echo "  DRIFT DETECTED in:$DRIFT_FILES"
    echo ""
    echo "  To update golden baseline, run:"
    echo "    just commit-golden-baseline"
    echo "============================================================="
    exit 1
fi
