# Migrate — Database migration management

# Run all pending migrations
migrate-up:
    @echo "=== Running migrations ==="
    @echo "Note: Migrations are applied automatically on server startup."
    @echo "For manual migration, run the API server with MIGRATE_ONLY=1"
    @echo ""
    @echo "Current migration status:"
    @just migrate-status

# Rollback last migration
migrate-down:
    @echo "=== Rollback not supported ==="
    @echo "Turso/libSQL migrations are additive by design."
    @echo "To revert, create a new migration that undoes the change."

# Show migration status
migrate-status:
    @echo "=== Migration Status ==="
    @echo "Database: servers/bff/web-bff/.data/web-bff.db"
    @echo ""
    @echo "Migrations are embedded in storage adapters:"
    @echo "  - storage_turso:    COUNTER_MIGRATION, CONVERSATION_MIGRATION, etc."
    @echo "  - storage_surrealdb: Programmatic table creation"
    @echo ""
    @echo "Tables in embedded Turso:"
    @if command -v sqlite3 >/dev/null 2>&1; then \
        sqlite3 servers/bff/web-bff/.data/web-bff.db ".tables" 2>/dev/null || echo "  (database not yet created)"; \
    else \
        echo "  sqlite3 not installed. Run: brew install sqlite"; \
    fi

# Create a new migration file
# Usage: just migrate-create add_counter_index
migrate-create NAME='':
    @if [ -z "{{NAME}}" ]; then \
        echo "Usage: just migrate-create add_counter_index"; \
        exit 1; \
    fi
    @TIMESTAMP=$(date +%Y%m%d%H%M%S); \
    MIGRATION_FILE="ops/migrations/api/${TIMESTAMP}_{{NAME}}.sql"; \
    mkdir -p ops/migrations/api; \
    touch "$MIGRATION_FILE"; \
    echo "Created migration: $MIGRATION_FILE"

# Reset database (dev only — DESTRUCTIVE)
migrate-reset:
    @echo "⚠️  WARNING: This will delete all data in the embedded database!"
    @echo "This is for development use only."
    @rm -f servers/bff/web-bff/.data/web-bff.db
    @echo "Database reset. Tables will be recreated on next server start."
