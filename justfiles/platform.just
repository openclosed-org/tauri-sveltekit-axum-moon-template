# Platform — platform model validation and catalog generation

# ── Model Validation ──────────────────────────────────────────

# Validate all platform models against JSON schemas
validate-platform:
    @echo "Validating platform models..."
    cargo run -p platform-validator -- --platform-dir platform

# Validate platform models with JSON output
validate-platform-json:
    cargo run -p platform-validator -- --platform-dir platform --format json

# Validate service ownership and state semantics alignment
validate-state MODE='warn':
    bun run scripts/validate-state.ts --mode {{MODE}}

# Validate workflow completeness beyond schema shape
validate-workflows MODE='warn':
    bun run scripts/validate-workflows.ts --mode {{MODE}}

# Generate platform catalog from models
gen-platform:
    @echo "Generating platform catalog..."
    cargo run -p platform-generator -- --platform-dir platform
    @echo "Platform catalog generated"

# Clean generated platform catalog
clean-platform:
    @echo "Cleaning generated platform catalog..."
    rm -rf platform/catalog/
    @echo "Catalog cleaned"

# Regenerate platform catalog (clean + generate)
regenerate-platform:
    just clean-platform
    just gen-platform

# ── Dependency Graph Validation ───────────────────────────────

# Validate dependency graph for cycles and broken references
validate-deps:
    @echo "Checking platform model dependency graph..."
    cargo run -p dependency-validator -- --platform-dir platform

# ── Contract Drift Detection ──────────────────────────────────

# Detect drift between platform models and contracts
validate-contracts:
    @echo "Checking contract drift..."
    cargo run -p contract-drift-detector -- --platform-dir platform

# ── Topology Validation ──────────────────────────────────────

# Validate deployment topology completeness
validate-topology:
    @echo "Checking deployment topology..."
    cargo run -p topology-validator -- --platform-dir platform

# Verify replay/rebuild reference hooks remain available
verify-replay MODE='warn':
    bun run scripts/verify-replay.ts --mode {{MODE}}

# Verify counter delivery admission path remains executable
verify-counter-delivery MODE='warn':
    bun run scripts/verify-counter-delivery.ts --mode {{MODE}}

# ── Security Validation ──────────────────────────────────────

# Validate security configuration
validate-security:
    @echo "Checking security configuration..."
    cargo run -p security-validator -- --platform-dir platform

# ── Observability Validation ─────────────────────────────────

# Validate observability configuration
validate-observability:
    @echo "Checking observability configuration..."
    cargo run -p observability-validator -- --platform-dir platform

# ── SDK Generation (Pending Implementation) ───────────────────

# Generate TypeScript and Rust SDKs from platform models
# TODO: Implement SDK generators in platform/generators/
gen-sdk:
    @echo "WARNING: SDK generation not yet implemented"
    @echo "  TypeScript SDK: packages/sdk/typescript/"
    @echo "  Rust SDK:       packages/sdk/rust/"
    @echo "  See: docs/architecture/repo-layout.md for planned structure"

# Clean generated SDKs
clean-sdk:
    @echo "Cleaning generated SDKs..."
    rm -rf packages/sdk/typescript/!(.gitkeep) packages/sdk/rust/!(.gitkeep) 2>/dev/null || true
    @echo "SDKs cleaned"

# ── Golden Baseline Verification ──────────────────────────────

# Verify generated artifacts match golden baseline
verify-generated:
    @bash scripts/verify-generated.sh

# Commit golden baselines for ALL generated artifacts
commit-golden-baseline:
    @bash scripts/commit-golden-baseline.sh

# ── Full Health Check ─────────────────────────────────────────

# Full platform doctor (all validations)
platform-doctor:
    @echo "Running platform health check..."
    @echo ""
    @echo "1. Validating models..."
    just validate-platform || (echo "Model validation failed"; exit 1)
    @echo ""
    @echo "2. Checking dependencies..."
    just validate-deps || (echo "Dependency check failed"; exit 1)
    @echo ""
    @echo "3. Checking state semantics..."
    just validate-state strict || (echo "State validation failed"; exit 1)
    @echo ""
    @echo "4. Checking workflows..."
    just validate-workflows strict || (echo "Workflow validation failed"; exit 1)
    @echo ""
    @echo "5. Checking contract drift..."
    just validate-contracts || true
    @echo ""
    @echo "6. Checking topology..."
    just validate-topology || (echo "Topology check failed"; exit 1)
    @echo ""
    @echo "7. Checking security..."
    just validate-security || (echo "Security check failed"; exit 1)
    @echo ""
    @echo "8. Checking observability..."
    just validate-observability || (echo "Observability check failed"; exit 1)
    @echo ""
    @echo "9. Verifying replay hooks..."
    just verify-replay strict || (echo "Replay verification failed"; exit 1)
    @echo ""
    @echo "10. Generating catalog..."
    just gen-platform || (echo "Catalog generation failed"; exit 1)
    @echo ""
    @echo "11. Verifying counter delivery admission..."
    just verify-counter-delivery strict || (echo "Counter delivery verification failed"; exit 1)
    @echo ""
    @echo "12. Verifying golden baseline..."
    just verify-generated || true
    @echo ""
    @echo "Platform health check passed"

# ── Inventory Commands ────────────────────────────────────────

# List all services defined in platform model
platform-services:
	@echo "Services defined in platform model:"
	@for f in platform/model/services/*.yaml; do \
		name=$$(basename "$$f" .yaml); \
		echo "  - $$name"; \
	done

# List all deployables defined in platform model
platform-deployables:
	@echo "Deployables defined in platform model:"
	@for f in platform/model/deployables/*.yaml; do \
		name=$$(basename "$$f" .yaml); \
		echo "  - $$name"; \
	done

# List all resources defined in platform model
platform-resources:
	@echo "Resources defined in platform model:"
	@for f in platform/model/resources/*.yaml; do \
		name=$$(basename "$$f" .yaml); \
		echo "  - $$name"; \
	done
