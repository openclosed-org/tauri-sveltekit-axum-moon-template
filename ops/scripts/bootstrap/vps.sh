#!/usr/bin/env bash
# vps.sh — Bare-metal VPS initialization script
#
# Installs the minimum toolchain needed to run this project on a fresh VPS.
# This is the ONE script to run on a new server before deploying.
#
# Usage:
#   curl -fsSL <repo-url>/ops/scripts/bootstrap/vps.sh | sudo bash
#   # Or locally:
#   sudo bash ops/scripts/bootstrap/vps.sh
#
# Installs:
#   - just (command runner)
#   - mise (tool version manager)
#   - docker (via official convenience script)
#   - kubectl + kustomize (for k3s deployment)
#   - age + sops (for secret decryption)
#   - git
#
# Supported OS: Ubuntu 22.04+, Debian 12+, Rocky Linux 9+

set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

info()  { echo -e "${GREEN}[INFO]${NC} $*"; }
warn()  { echo -e "${YELLOW}[WARN]${NC} $*"; }
error() { echo -e "${RED}[ERROR]${NC} $*"; }

# ── Pre-flight ────────────────────────────────────────────────
if [[ $EUID -ne 0 ]]; then
    error "This script must be run as root (use sudo)"
    exit 1
fi

# Detect OS
if [[ -f /etc/os-release ]]; then
    . /etc/os-release
    OS=$ID
else
    error "Cannot detect OS"
    exit 1
fi

info "Detected OS: $OS $VERSION_ID"

# ── Package manager abstraction ───────────────────────────────
install_packages() {
    case "$OS" in
        ubuntu|debian)
            apt-get update
            apt-get install -y --no-install-recommends "$@"
            ;;
        rocky|centos|rhel|fedora)
            dnf install -y "$@"
            ;;
        *)
            error "Unsupported OS: $OS"
            exit 1
            ;;
    esac
}

# ── Install git ───────────────────────────────────────────────
if command -v git &>/dev/null; then
    info "git already installed: $(git --version)"
else
    info "Installing git..."
    install_packages git
fi

# ── Install just (command runner) ────────────────────────────
if command -v just &>/dev/null; then
    info "just already installed: $(just --version)"
else
    info "Installing just..."
    case "$OS" in
        ubuntu|debian)
            # Use the official installer
            curl --proto '=https' --tlsv1.2 -sSf https://just.systems/install.sh | bash -s -- --to /usr/local/bin
            ;;
        rocky|centos|rhel|fedora)
            curl --proto '=https' --tlsv1.2 -sSf https://just.systems/install.sh | bash -s -- --to /usr/local/bin
            ;;
    esac
fi

# ── Install mise (tool version manager) ──────────────────────
if command -v mise &>/dev/null; then
    info "mise already installed: $(mise --version)"
else
    info "Installing mise..."
    curl https://mise.run | sh
    # Add to profile for interactive shells
    echo 'eval "$(~/.local/bin/mise activate bash)"' >> /etc/profile.d/mise.sh 2>/dev/null || true
fi

# ── Install Podman (daemonless, rootless container runtime) ──
if command -v podman &>/dev/null; then
    info "Podman already installed: $(podman --version)"
else
    info "Installing Podman..."
    case "$OS" in
        ubuntu|debian)
            # Use official Kubic repository (or default apt if available)
            install_packages podman podman-compose
            ;;
        rocky|centos|rhel|fedora)
            dnf install -y podman podman-compose
            ;;
    esac
    # Enable podman socket for rootless API access
    systemctl --user enable podman.socket 2>/dev/null || true
    systemctl --user start podman.socket 2>/dev/null || true
fi

# ── Install kubectl ──────────────────────────────────────────
if command -v kubectl &>/dev/null; then
    info "kubectl already installed: $(kubectl version --client --short 2>/dev/null || kubectl version --client)"
else
    info "Installing kubectl..."
    curl -LO "https://dl.k8s.io/release/$(curl -L -s https://dl.k8s.io/release/stable.txt)/bin/linux/amd64/kubectl"
    install -o root -g root -m 0755 kubectl /usr/local/bin/kubectl
    rm -f kubectl
fi

# ── Install kustomize ────────────────────────────────────────
if command -v kustomize &>/dev/null; then
    info "kustomize already installed: $(kustomize version)"
else
    info "Installing kustomize..."
    curl -s "https://raw.githubusercontent.com/kubernetes-sigs/kustomize/master/hack/install_kustomize.sh" | bash -s -- /usr/local/bin
fi

# ── Install age (encryption) ─────────────────────────────────
if command -v age &>/dev/null; then
    info "age already installed: $(age -version)"
else
    info "Installing age..."
    case "$OS" in
        ubuntu|debian)
            install_packages age
            ;;
        rocky|centos|rhel|fedora)
            # age may not be in default repos for RHEL-based
            # Install from GitHub releases
            ARCH=$(uname -m)
            if [[ "$ARCH" == "x86_64" ]]; then
                ARCH_TAG="amd64"
            else
                ARCH_TAG="arm64"
            fi
            LATEST=$(curl -s https://api.github.com/repos/FiloSottile/age/releases/latest | grep tag_name | cut -d'"' -f4)
            curl -sL "https://github.com/FiloSottile/age/releases/download/${LATEST}/age-${LATEST}-linux-${ARCH_TAG}.tar.gz" | tar xz
            mv "age/age" /usr/local/bin/
            mv "age/age-keygen" /usr/local/bin/
            rm -rf age
            ;;
    esac
fi

# ── Install sops ─────────────────────────────────────────────
if command -v sops &>/dev/null; then
    info "sops already installed: $(sops --version 2>&1 | head -1)"
else
    info "Installing sops..."
    LATEST=$(curl -s https://api.github.com/repos/getsops/sops/releases/latest | grep tag_name | cut -d'"' -f4)
    curl -sLo /usr/local/bin/sops "https://github.com/getsops/sops/releases/download/${LATEST}/sops-${LATEST}.linux.amd64"
    chmod +x /usr/local/bin/sops
fi

# ── Post-install summary ─────────────────────────────────────
echo ""
info "═══════════════════════════════════════════════════════"
info "  VPS bootstrap complete!"
info "═══════════════════════════════════════════════════════"
info ""
info "  Installed tools:"
info "    just:    $(just --version 2>/dev/null || echo 'installed')"
info "    podman:  $(podman --version 2>/dev/null || echo 'installed')"
info "    kubectl: $(kubectl version --client --short 2>/dev/null || kubectl version --client 2>/dev/null | head -1 || echo 'installed')"
info "    kustomize: $(kustomize version 2>/dev/null || echo 'installed')"
info "    age:     $(age -version 2>/dev/null || echo 'installed')"
info "    sops:    $(sops --version 2>&1 | head -1 2>/dev/null || echo 'installed')"
info ""
info "  Next steps:"
info "    1. Clone the repository"
info "    2. Run: just deploy bootstrap-k3s  (for k3s cluster)"
info "    3. Run: just deploy prod"
info ""
info "═══════════════════════════════════════════════════════"
