#!/usr/bin/env bash
# commit-golden-baseline.sh — Generate and commit golden baseline for platform artifacts
#
# Usage: just commit-golden-baseline
#
# This script:
# 1. Generates fresh platform catalog
# 2. Copies to verification/golden/generated-platform/
# 3. Reports what was committed

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
GOLDEN_DIR="$PROJECT_ROOT/verification/golden/generated-platform"

echo "=== Committing golden platform baseline ==="
echo ""

echo "1. Generating fresh platform catalog..."
cargo run -p platform-generator --quiet -- --platform-dir "$PROJECT_ROOT/platform" --output-dir "$PROJECT_ROOT/platform/catalog"

echo "2. Copying to golden baseline directory..."
mkdir -p "$GOLDEN_DIR"
cp "$PROJECT_ROOT/platform/catalog/"* "$GOLDEN_DIR/"

echo ""
echo "3. Golden baseline committed:"
for file in "$GOLDEN_DIR"/*; do
    filename=$(basename "$file")
    lines=$(wc -l < "$file")
    echo "   - $filename ($lines lines)"
done

echo ""
echo "Next step: git add $GOLDEN_DIR && git commit -m 'chore: commit golden platform baseline'"
