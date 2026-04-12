#!/usr/bin/env bash
# Bootstrap script for local development infrastructure
#
# Usage:
#   bash infra/local/scripts/bootstrap.sh [up|down|restart|status|logs]
#
# This script manages the local development infrastructure using docker compose.

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
COMPOSE_FILE="infra/docker/compose/core.yaml"
PROJECT_NAME="tauri-sveltekit-dev"

# Functions
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

check_docker() {
    if ! command -v docker &> /dev/null && ! command -v podman &> /dev/null; then
        log_error "Neither docker nor podman found. Please install one of them."
        exit 1
    fi

    if command -v docker &> /dev/null; then
        COMPOSE_CMD="docker compose"
    else
        COMPOSE_CMD="podman compose"
    fi
}

start_services() {
    log_info "Starting local development infrastructure..."
    
    if [ ! -f "$COMPOSE_FILE" ]; then
        log_error "Compose file not found: $COMPOSE_FILE"
        exit 1
    fi

    $COMPOSE_CMD -f "$COMPOSE_FILE" -p "$PROJECT_NAME" up -d
    
    log_info "Waiting for services to be healthy..."
    sleep 5
    
    # Check service health
    local unhealthy=0
    for container in $("COMPOSE_CMD" -f "$COMPOSE_FILE" -p "$PROJECT_NAME" ps -q); do
        local health=$($COMPOSE_CMD -f "$COMPOSE_FILE" -p "$PROJECT_NAME" ps --format json | grep -o '"Health":"[^"]*"' | cut -d'"' -f4)
        if [[ "$health" == "unhealthy" ]]; then
            unhealthy=1
        fi
    done

    if [ $unhealthy -eq 0 ]; then
        log_success "All services started successfully!"
        print_connection_info
    else
        log_warning "Some services may not be healthy. Check logs with: $0 logs"
    fi
}

stop_services() {
    log_info "Stopping local development infrastructure..."
    $COMPOSE_CMD -f "$COMPOSE_FILE" -p "$PROJECT_NAME" down
    log_success "All services stopped."
}

restart_services() {
    stop_services
    start_services
}

status_services() {
    log_info "Service status:"
    echo ""
    $COMPOSE_CMD -f "$COMPOSE_FILE" -p "$PROJECT_NAME" ps
    echo ""
    
    # Show port mappings
    log_info "Port mappings:"
    echo ""
    echo "  Turso/libSQL:    http://localhost:8080 (HTTP), grpc://localhost:5001 (gRPC)"
    echo "  NATS:            nats://localhost:4222, http://localhost:8222 (monitoring)"
    echo "  Valkey:          redis://localhost:6379"
    echo "  MinIO API:       http://localhost:9000"
    echo "  MinIO Console:   http://localhost:9001 (minioadmin/minioadmin)"
    echo ""
}

logs_services() {
    $COMPOSE_CMD -f "$COMPOSE_FILE" -p "$PROJECT_NAME" logs -f
}

print_connection_info() {
    echo ""
    log_info "Connection Information:"
    echo ""
    echo "  📊 Turso/libSQL:"
    echo "     HTTP API:  http://localhost:8080"
    echo "     gRPC:      grpc://localhost:5001"
    echo ""
    echo "  📨 NATS:"
    echo "     Client:    nats://localhost:4222"
    echo "     Monitor:   http://localhost:8222"
    echo ""
    echo "  💾 Valkey (Redis-compatible):"
    echo "     URL:       redis://localhost:6379"
    echo "     CLI:       redis-cli -h localhost -p 6379"
    echo ""
    echo "  📦 MinIO (S3-compatible):"
    echo "     API:       http://localhost:9000"
    echo "     Console:   http://localhost:9001"
    echo "     Access Key: minioadmin"
    echo "     Secret Key: minioadmin"
    echo "     Buckets:   uploads, backups, temp"
    echo ""
    log_info "To stop all services: $0 down"
}

# Main script
check_docker

ACTION="${1:-up}"

case "$ACTION" in
    up|start)
        start_services
        ;;
    down|stop)
        stop_services
        ;;
    restart)
        restart_services
        ;;
    status)
        status_services
        ;;
    logs)
        logs_services
        ;;
    *)
        echo "Usage: $0 {up|down|restart|status|logs}"
        echo ""
        echo "Commands:"
        echo "  up, start    - Start all services"
        echo "  down, stop   - Stop all services"
        echo "  restart      - Restart all services"
        echo "  status       - Show service status"
        echo "  logs         - Follow service logs"
        exit 1
        ;;
esac
