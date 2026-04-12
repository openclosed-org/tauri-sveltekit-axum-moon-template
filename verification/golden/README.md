# Golden Baseline

This directory contains the golden baseline for generated artifacts.
After running generation commands, compare output against this baseline to detect drift.

## Contents

### generated-sdk/
Golden baseline for SDK generation output.
After `just gen-sdk`, diff against this directory.

### rendered-manifests/
Golden baseline for rendered infrastructure manifests.
After `just render-manifests`, diff against this directory.

### contracts/
Golden baseline for generated contract types.
After `just typegen`, diff `packages/contracts/generated/` against this directory.

## Usage

```bash
# Generate all artifacts
just gen-platform
just typegen

# Check for drift
just verify-generated
```

## Initial Baseline

The initial baseline was created during Phase 4 (verification/ setup).
To update the baseline after intentional changes:

```bash
# Regenerate artifacts
just gen-platform
just typegen

# Copy to golden directory
cp -r platform/catalog/ verification/golden/generated-platform/
cp -r packages/contracts/generated/ verification/golden/generated-contracts/

# Commit the change
git add verification/golden/
git commit -m "chore: update golden baseline"
```
