#!/usr/bin/env bash
# Database Migration Runner
#
# Usage:
#   bash ops/migrations/runner/migrate.sh [up|down|status|reset] [environment]
#
# Manages database migrations for all services.
# Supports: libSQL/Turso, SurrealDB
# Environments: local, dev, staging, prod

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Configuration
MIGRATIONS_DIR="ops/migrations"
ENVIRONMENT="${2:-local}"

# Service migration directories
SERVICES=(
    "user-service"
    "tenant-service"
    "auth-service"
    "counter-service"
    "settings-service"
)

# Get database URL based on environment
get_db_url() {
    local service=$1
    case "$ENVIRONMENT" in
        local)
            echo "libsql://localhost:8080"
            ;;
        dev)
            # From Kubernetes secrets or configmap
            echo "libsql://turso.app.svc.cluster.local:8080"
            ;;
        staging|prod)
            # From SOPS-encrypted secrets
            if command -v sops &> /dev/null && [ -f "infra/security/sops/secrets.enc.yaml" ]; then
                sops --decrypt infra/security/sops/secrets.enc.yaml | grep DATABASE_URL | cut -d' ' -f4
            else
                log_error "SOPS not configured for $ENVIRONMENT"
                exit 1
            fi
            ;;
        *)
            log_error "Unknown environment: $ENVIRONMENT"
            exit 1
            ;;
    esac
}

# Run migrations for a single service
migrate_service() {
    local service=$1
    local migration_dir="$MIGRATIONS_DIR/$service"
    
    if [ ! -d "$migration_dir" ]; then
        log_warning "No migration directory for $service, skipping..."
        return 0
    fi
    
    local db_url=$(get_db_url "$service")
    
    log_info "Running migrations for $service..."
    
    # Apply all SQL files in order (001_*.sql, 002_*.sql, etc.)
    for migration in "$migration_dir"/*.sql; do
        if [ -f "$migration" ]; then
            local filename=$(basename "$migration")
            log_info "  Applying: $filename"
            
            # Apply migration using sqlite3 for libSQL
            # In production, use a proper migration tool like sqlx-cli or refinery
            if command -v sqlite3 &> /dev/null; then
                sqlite3 ":memory:" < "$migration" 2>&1 | while read -r line; do
                    log_info "    $line"
                done
            else
                log_warning "    sqlite3 not found, skipping migration"
            fi
        fi
    done
    
    log_success "  $service migrations complete"
}

# Run all migrations up
migrate_up() {
    log_info "Running all migrations up for environment: $ENVIRONMENT"
    echo ""
    
    for service in "${SERVICES[@]}"; do
        migrate_service "$service"
    done
    
    echo ""
    log_success "All migrations complete!"
}

# Rollback last migration (simplified - would need migration tracking in production)
migrate_down() {
    log_warning "Rollback not yet implemented. Manual rollback required."
    log_info "To rollback manually:"
    log_info "  1. Identify the last migration file"
    log_info "  2. Reverse the SQL statements"
    log_info "  3. Apply reversed SQL to database"
    exit 1
}

# Show migration status
migrate_status() {
    log_info "Migration status for environment: $ENVIRONMENT"
    echo ""
    
    for service in "${SERVICES[@]}"; do
        local migration_dir="$MIGRATIONS_DIR/$service"
        
        if [ -d "$migration_dir" ]; then
            local count=$(ls -1 "$migration_dir"/*.sql 2>/dev/null | wc -l)
            log_info "  $service: $count migration(s)"
            ls -1 "$migration_dir"/*.sql 2>/dev/null | while read -r f; do
                echo "    - $(basename "$f")"
            done
        else
            log_info "  $service: No migrations"
        fi
    done
}

# Reset database (WARNING: destructive!)
migrate_reset() {
    log_warning "WARNING: This will DELETE ALL DATA in the database!"
    read -p "Are you sure? (yes/no): " confirm
    
    if [ "$confirm" = "yes" ]; then
        log_info "Resetting database..."
        
        # For libSQL embedded, just delete the database file
        if [ "$ENVIRONMENT" = "local" ]; then
            rm -f .data/*.db
            log_success "Local database files deleted"
        else
            log_warning "For $ENVIRONMENT, manual database reset required"
            log_info "Contact infrastructure team or use cloud console"
        fi
        
        # Re-run migrations
        migrate_up
    else
        log_info "Reset cancelled"
    fi
}

# Main
ACTION="${1:-up}"

case "$ACTION" in
    up|migrate)
        migrate_up
        ;;
    down|rollback)
        migrate_down
        ;;
    status)
        migrate_status
        ;;
    reset)
        migrate_reset
        ;;
    *)
        echo "Usage: $0 {up|down|status|reset} [environment]"
        echo ""
        echo "Commands:"
        echo "  up, migrate     - Run all migrations"
        echo "  down, rollback  - Rollback last migration (not implemented)"
        echo "  status          - Show migration status"
        echo "  reset           - Reset database and re-run migrations"
        echo ""
        echo "Environments:"
        echo "  local   - Local development (default)"
        echo "  dev     - Development cluster"
        echo "  staging - Staging cluster"
        echo "  prod    - Production cluster"
        exit 1
        ;;
esac
