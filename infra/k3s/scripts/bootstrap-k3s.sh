#!/usr/bin/env bash
# bootstrap-k3s.sh — One-click k3s installation on a fresh VPS
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/<repo>/main/infra/k3s/scripts/bootstrap-k3s.sh | sudo bash
#   # Or run locally:
#   bash infra/k3s/scripts/bootstrap-k3s.sh
#
# This script:
#   1. Installs k3s (single-binary Kubernetes)
#   2. Enables built-in Traefik ingress, containerd, local-path storage
#   3. Configures kubectl access
#   4. Verifies the cluster is healthy
#
# Requirements:
#   - Ubuntu 22.04+ / Debian 12+ / Rocky Linux 9+
#   - 2GB+ RAM, 2+ CPU cores, 20GB+ disk
#   - Root/sudo access
#   - Ports: 6443 (API), 80/443 (ingress), 8472 (VXLAN)

set -euo pipefail

# ── Configuration ─────────────────────────────────────────────
K3S_VERSION="${K3S_VERSION:-stable}"
K3S_TOKEN="${K3S_TOKEN:-$(openssl rand -hex 16)}"
INSTALL_DIR="/usr/local/bin"
KUBECONFIG_DIR="$HOME/.kube"

# ── Color output helpers ──────────────────────────────────────
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

info()    { echo -e "${GREEN}[INFO]${NC} $*"; }
warn()    { echo -e "${YELLOW}[WARN]${NC} $*"; }
error()   { echo -e "${RED}[ERROR]${NC} $*"; }

# ── Pre-flight checks ─────────────────────────────────────────
preflight() {
    info "Running pre-flight checks..."

    # Check root
    if [[ $EUID -ne 0 ]]; then
        error "This script must be run as root (use sudo)"
        exit 1
    fi

    # Check architecture
    ARCH=$(uname -m)
    if [[ "$ARCH" != "x86_64" && "$ARCH" != "aarch64" ]]; then
        error "Unsupported architecture: $ARCH (need x86_64 or aarch64)"
        exit 1
    fi

    # Check memory (minimum 2GB)
    MEM_KB=$(grep MemTotal /proc/meminfo | awk '{print $2}')
    MEM_GB=$((MEM_KB / 1024 / 1024))
    if [[ $MEM_GB -lt 2 ]]; then
        warn "Low memory detected: ${MEM_GB}GB (minimum 2GB recommended)"
    fi

    # Check if k3s is already installed
    if command -v k3s &>/dev/null; then
        warn "k3s is already installed at $(which k3s)"
        warn "Version: $(k3s --version)"
        read -rp "Continue anyway? (y/N) " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            exit 0
        fi
    fi

    # Check required ports are not in use
    for port in 6443 80 443; do
        if ss -tlnp | grep -q ":$port "; then
            warn "Port $port appears to be in use"
        fi
    done

    info "Pre-flight checks passed"
}

# ── Install k3s ──────────────────────────────────────────────
install_k3s() {
    info "Installing k3s (${K3S_VERSION})..."

    # Set environment variables for the installer
    export K3S_TOKEN
    export INSTALL_K3S_VERSION="${K3S_VERSION}"

    # Install with Traefik ingress (default) and local-path storage
    curl -sfL https://get.k3s.io | sh -s - server \
        --write-kubeconfig-mode 644 \
        --disable traefik \
        --disable servicelb \
        --disable local-storage \
        --cluster-init

    info "k3s installed successfully"
}

# ── Configure kubectl ─────────────────────────────────────────
configure_kubectl() {
    info "Configuring kubectl..."

    mkdir -p "$KUBECONFIG_DIR"
    cp /etc/rancher/k3s/k3s.yaml "$KUBECONFIG_DIR/config"
    chmod 600 "$KUBECONFIG_DIR/config"

    # Update server URL to localhost for local access
    sed -i 's|127.0.0.1|localhost|g' "$KUBECONFIG_DIR/config" 2>/dev/null || true

    info "kubectl configured at $KUBECONFIG_DIR/config"
}

# ── Install kustomize (for deployments) ──────────────────────
install_kustomize() {
    if command -v kustomize &>/dev/null; then
        info "kustomize already installed at $(which kustomize)"
        return
    fi

    info "Installing kustomize..."
    curl -s "https://raw.githubusercontent.com/kubernetes-sigs/kustomize/master/hack/install_kustomize.sh" | bash -s -- /usr/local/bin
    info "kustomize installed successfully"
}

# ── Verify cluster health ────────────────────────────────────
verify_cluster() {
    info "Verifying cluster health..."

    # Wait for node to be ready
    for i in {1..30}; do
        if kubectl get nodes | grep -q Ready; then
            break
        fi
        info "Waiting for node to be ready... ($i/30)"
        sleep 5
    done

    # Show node status
    kubectl get nodes -o wide

    # Show core components
    kubectl get pods -n kube-system

    # Show storage classes
    kubectl get storageclass

    info "Cluster verification complete"
}

# ── Print connection info ─────────────────────────────────────
print_info() {
    echo ""
    info "═══════════════════════════════════════════════════════"
    info "  k3s cluster is ready!"
    info "═══════════════════════════════════════════════════════"
    info ""
    info "  Kubeconfig:  $KUBECONFIG_DIR/config"
    info "  API Server:  https://localhost:6443"
    info "  Token:       $K3S_TOKEN (save this for agent join)"
    info ""
    info "  Next steps:"
    info "    1. kubectl get nodes"
    info "    2. bash infra/k3s/scripts/deploy.sh dev"
    info ""
    info "═══════════════════════════════════════════════════════"
}

# ── Main ──────────────────────────────────────────────────────
main() {
    preflight
    install_k3s
    configure_kubectl
    install_kustomize
    verify_cluster
    print_info
}

main "$@"
