#!/usr/bin/env bash
# deploy.sh — kubectl/kustomize deployment entrypoint for k3s
#
# Usage:
#   bash infra/k3s/scripts/deploy.sh <environment>
#
# Environments:
#   dev       — Development (minimal resources, debug logging)
#   staging   — Staging (production-like config)
#   prod      — Production (HPA, multi-replica, strict quotas)
#
# Examples:
#   bash infra/k3s/scripts/deploy.sh dev
#   bash infra/k3s/scripts/deploy.sh prod --dry-run=client
#   bash infra/k3s/scripts/deploy.sh prod --diff

set -euo pipefail

# ── Configuration ─────────────────────────────────────────────
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
INFRA_DIR="$(dirname "$(dirname "$SCRIPT_DIR")")"
K3S_DIR="$INFRA_DIR/k3s"
OVERLAY_DIR="$K3S_DIR/overlays"

# ── Color output helpers ──────────────────────────────────────
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

info()    { echo -e "${GREEN}[INFO]${NC} $*"; }
warn()    { echo -e "${YELLOW}[WARN]${NC} $*"; }
error()   { echo -e "${RED}[ERROR]${NC} $*"; }
step()    { echo -e "${BLUE}[STEP]${NC} $*"; }

# ── Usage ─────────────────────────────────────────────────────
usage() {
    cat <<EOF
Usage: $(basename "$0") <environment> [kubectl-args...]

Environments:
  dev       Development overlay
  staging   Staging overlay
  prod      Production overlay

Examples:
  $(basename "$0") dev
  $(basename "$0") staging
  $(basename "$0") prod
  $(basename "$0") prod --dry-run=client
  $(basename "$0") prod --diff

Options:
  --help    Show this help message
EOF
    exit 0
}

# ── Validate environment ─────────────────────────────────────
validate_env() {
    local env="$1"

    if [[ ! -d "$OVERLAY_DIR/$env" ]]; then
        error "Unknown environment: $env"
        error "Available environments: $(ls -1 "$OVERLAY_DIR" | tr '\n' ' ')"
        exit 1
    fi

    if [[ ! -f "$OVERLAY_DIR/$env/kustomization.yaml" ]]; then
        error "No kustomization.yaml found in $OVERLAY_DIR/$env"
        exit 1
    fi
}

# ── Pre-flight checks ─────────────────────────────────────────
preflight() {
    info "Running pre-flight checks..."

    # Check kubectl
    if ! command -v kubectl &>/dev/null; then
        error "kubectl is not installed. Install it: https://kubernetes.io/docs/tasks/tools/"
        exit 1
    fi

    # Check kustomize
    if ! command -v kustomize &>/dev/null; then
        error "kustomize is not installed. Install it: https://kubectl.docs.kubernetes.io/installation/kustomize/"
        exit 1
    fi

    # Check cluster connectivity
    if ! kubectl cluster-info &>/dev/null; then
        error "Cannot connect to Kubernetes cluster"
        error "Run bootstrap first: bash $K3S_DIR/scripts/bootstrap-k3s.sh"
        exit 1
    fi

    info "Pre-flight checks passed"
}

# ── Build and render manifest ─────────────────────────────────
build_manifest() {
    local env="$1"
    info "Building manifest for environment: $env"

    kustomize build "$OVERLAY_DIR/$env"
}

# ── Deploy ────────────────────────────────────────────────────
deploy() {
    local env="$1"
    shift
    local extra_args=("$@")

    step "Deploying to $env..."

    # Show what will be applied
    info "Rendering manifest..."
    local manifest
    manifest=$(kustomize build "$OVERLAY_DIR/$env")

    # Count resources
    local resource_count
    resource_count=$(echo "$manifest" | grep -c "^kind:" || true)
    info "Found $resource_count resources to apply"

    # Apply
    if [[ " ${extra_args[*]:-} " =~ "--dry-run" ]]; then
        info "Dry run mode — not applying changes"
        kubectl apply -k "$OVERLAY_DIR/$env" "${extra_args[@]}"
    else
        kubectl apply -k "$OVERLAY_DIR/$env" "${extra_args[@]}"
    fi

    info "Deployment initiated"
}

# ── Post-deploy verification ──────────────────────────────────
verify() {
    local env="$1"

    step "Verifying deployment..."

    # Wait for deployments to be ready
    info "Waiting for deployments to be ready..."
    kubectl wait --for=condition=available --timeout=120s deployment --all -n app 2>/dev/null || {
        warn "Some deployments may not be ready yet. Check with: kubectl get pods -n app"
    }

    # Show pod status
    info "Pod status:"
    kubectl get pods -n app -o wide

    # Show services
    info "Services:"
    kubectl get svc -n app

    info "Verification complete"
}

# ── Main ──────────────────────────────────────────────────────
main() {
    # Parse arguments
    if [[ $# -lt 1 || "$1" == "--help" || "$1" == "-h" ]]; then
        usage
    fi

    local env="$1"
    shift
    local extra_args=("$@")

    validate_env "$env"
    preflight
    deploy "$env" "${extra_args[@]}"
    verify "$env"

    echo ""
    info "═══════════════════════════════════════════════════════"
    info "  Deployment to '$env' complete!"
    info ""
    info "  Useful commands:"
    info "    kubectl get pods -n app"
    info "    kubectl logs -n app -l app.kubernetes.io/name=api-server -f"
    info "    kubectl port-forward -n app svc/web-server 3000:80"
    info "═══════════════════════════════════════════════════════"
}

main "$@"
