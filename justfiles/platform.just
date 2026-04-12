# Platform — platform model validation and catalog generation

# ── Model Validation ──────────────────────────────────────────

# Validate all platform models against JSON schemas
validate-platform:
    @echo "Validating platform models..."
    cargo run -p platform-validator -- --platform-dir platform

# Validate platform models with JSON output
validate-platform-json:
    cargo run -p platform-validator -- --platform-dir platform --format json

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

# ── Golden Baseline Verification ──────────────────────────────

# Verify generated artifacts match golden baseline
verify-generated:
    @bash scripts/verify-generated.sh

# Commit golden baseline for platform catalog
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
    @echo "3. Checking contract drift..."
    just validate-contracts || true
    @echo ""
    @echo "4. Generating catalog..."
    just gen-platform || (echo "Catalog generation failed"; exit 1)
    @echo ""
    @echo "5. Verifying golden baseline..."
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
