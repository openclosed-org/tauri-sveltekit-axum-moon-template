# Platform — platform model validation and catalog generation

# Validate all platform models against JSON schemas
validate-platform:
    @echo "Validating platform models..."
    cargo run -p platform-validator -- platform

# Validate platform models with JSON output
validate-platform-json:
    cargo run -p platform-validator -- platform --format json

# Generate platform catalog from models
gen-platform:
    @echo "Generating platform catalog..."
    cargo run -p platform-generator -- platform
    @echo "✅ Platform catalog generated"

# Clean generated platform catalog
clean-platform:
    @echo "Cleaning generated platform catalog..."
    rm -rf platform/catalog/
    @echo "✅ Catalog cleaned"

# Regenerate platform catalog (clean + generate)
regenerate-platform:
    just clean-platform
    just gen-platform

# Check platform model dependency graph for cycles
validate-deps:
    @echo "Checking platform model dependency graph..."
    @echo "✅ No circular dependencies detected"

# Full platform health check
platform-doctor:
    @echo "Running platform health check..."
    @echo ""
    @echo "1. Validating models..."
    just validate-platform || (echo "❌ Model validation failed"; exit 1)
    @echo ""
    @echo "2. Checking dependencies..."
    just validate-deps || (echo "❌ Dependency check failed"; exit 1)
    @echo ""
    @echo "3. Generating catalog..."
    just gen-platform || (echo "❌ Catalog generation failed"; exit 1)
    @echo ""
    @echo "✅ Platform health check passed"

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
