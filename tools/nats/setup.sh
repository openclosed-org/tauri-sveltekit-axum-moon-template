#!/usr/bin/env bash
# NATS Server local binary setup
# Downloads nats-server if not present in tools/nats/
# Supports: macOS (arm64/amd64), Linux (arm64/amd64), Windows (amd64 via Git Bash/WSL)
set -euo pipefail

NATS_VERSION="2.10.22"
NATS_DIR="$(cd "$(dirname "$0")" && pwd)"
NATS_BIN="$NATS_DIR/nats-server"

if [ -f "$NATS_BIN" ] || [ -f "$NATS_BIN.exe" ]; then
  echo "✓ nats-server already present: $NATS_BIN"
  exit 0
fi

# Detect OS
case "$(uname -s 2>/dev/null || echo Windows)" in
  Darwin)  OS="darwin" ;;
  Linux*)  OS="linux" ;;
  MINGW*|MSYS*|CYGWIN*|Windows*)
           OS="windows" ;;
  *)       echo "Unsupported OS: $(uname -s)"; exit 1 ;;
esac

# Detect ARCH
case "$(uname -m 2>/dev/null || echo unknown)" in
  arm64|aarch64) ARCH="arm64" ;;
  x86_64|amd64)  ARCH="amd64" ;;
  *)
    # Fallback: try PROCESSOR_ARCHITECTURE on Windows
    case "${PROCESSOR_ARCHITECTURE:-}" in
      AMD64) ARCH="amd64" ;;
      ARM64) ARCH="arm64" ;;
      *)     echo "Unsupported arch: $(uname -m)"; exit 1 ;;
    esac
    ;;
esac

echo "Platform: ${OS}/${ARCH}"

if [ "$OS" = "windows" ]; then
  # Windows: download zip, extract .exe
  FILENAME="nats-server-v${NATS_VERSION}-${OS}-${ARCH}.zip"
  URL="https://github.com/nats-io/nats-server/releases/download/v${NATS_VERSION}/${FILENAME}"
  echo "Downloading nats-server v${NATS_VERSION} for ${OS}/${ARCH}..."
  curl -L -o "$NATS_DIR/$FILENAME" "$URL"
  unzip -o "$NATS_DIR/$FILENAME" -d "$NATS_DIR"
  rm "$NATS_DIR/$FILENAME"
  echo "✓ nats-server installed: $NATS_DIR/nats-server.exe"
else
  # macOS/Linux: download tar.gz
  FILENAME="nats-server-v${NATS_VERSION}-${OS}-${ARCH}.tar.gz"
  URL="https://github.com/nats-io/nats-server/releases/download/v${NATS_VERSION}/${FILENAME}"
  echo "Downloading nats-server v${NATS_VERSION} for ${OS}/${ARCH}..."
  curl -L -o "$NATS_DIR/$FILENAME" "$URL"
  tar -xzf "$NATS_DIR/$FILENAME" -C "$NATS_DIR" --strip-components=1 "nats-server-v${NATS_VERSION}-${OS}-${ARCH}/nats-server"
  rm "$NATS_DIR/$FILENAME"
  chmod +x "$NATS_BIN"
  echo "✓ nats-server installed: $NATS_BIN"
fi
