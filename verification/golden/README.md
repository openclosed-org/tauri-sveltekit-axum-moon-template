# Golden Baseline

This directory contains the golden baseline for generated and reference artifacts.
After running generation or replay-related verification commands, compare output against this baseline to detect drift.

## Contents

### contracts/
Golden baseline for generated contract types.
After `just typegen`, diff `packages/contracts/generated/` against this directory.

### generated-platform/
Golden baseline for generated platform catalog output.
After `just generate-platform-catalog`, diff platform outputs against this directory.

### replay lanes
Replay and rebuild references are anchored by:

1. `workers/projector/`
2. `workers/outbox-relay/`
3. `services/counter-service/`

These are not copied here wholesale, but `just verify-replay` treats them as required golden lanes.

## Usage

```bash
# Generate all artifacts
just generate-platform-catalog
just typegen
just verify-replay strict

# Check for drift
just verify-generated-artifacts
```

## Initial Baseline

The baseline is currently a harness-focused snapshot while the distributed reference set converges.
To update the baseline after intentional changes:

```bash
# Regenerate artifacts
just generate-platform-catalog
just typegen

# Refresh generated platform and contract outputs, then update the matching golden files.
```
