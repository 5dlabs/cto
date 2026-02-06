#!/bin/bash
# Download talosctl binary for the current platform
# Usage: ./download-talosctl.sh [--version VERSION] [--arch ARCH] [--os OS] [--dest DIR]

set -e

TALOS_VERSION="${TALOS_VERSION:-v1.10.4}"
ARCH="${ARCH:-$(uname -m)}"
OS="${OS:-$(uname -s | tr '[:upper:]' '[:lower:]')}"
DEST="${DEST:-/usr/local/bin}"

# Detect architecture
case "$ARCH" in
    x86_64) ARCH="amd64" ;;
    aarch64|arm64) ARCH="arm64" ;;
    *)
        echo "❌ Unsupported architecture: $ARCH"
        exit 1
        ;;
esac

# Detect OS mapping
case "$OS" in
    darwin) OS="darwin" ;;
    linux) OS="linux" ;;
    *)
        echo "❌ Unsupported OS: $OS"
        exit 1
        ;;
esac

TALOSCTL_URL="https://github.com/siderolabs/talos/releases/download/${TALOS_VERSION}/talosctl-${OS}-${ARCH}"
TALOSCTL_PATH="${DEST}/talosctl"

echo "⬇️  Downloading talosctl ${TALOS_VERSION} for ${OS}-${ARCH}..."
echo "   URL: ${TALOSCTL_URL}"

# Download with retry (follow redirects for GitHub releases)
MAX_RETRIES=3
for i in $(seq 1 $MAX_RETRIES); do
    if curl -L -o "${TALOSCTL_PATH}" "${TALOSCTL_URL}" 2>/dev/null; then
        chmod +x "${TALOSCTL_PATH}"
        echo "✅ talosctl installed to ${TALOSCTL_PATH}"

        # Verify checksum
        echo "🔍 Verifying checksum..."
        CHECKSUM_URL="https://github.com/siderolabs/talos/releases/download/${TALOS_VERSION}/sha256sum.txt"
        EXPECTED_CHECKSUM=$(curl -sL "${CHECKSUM_URL}" | grep "talosctl-${OS}-${ARCH}$" | awk '{print $1}')
        if command -v sha256sum &>/dev/null; then
            ACTUAL_CHECKSUM=$(sha256sum "${TALOSCTL_PATH}" | awk '{print $1}')
        elif command -v shasum &>/dev/null; then
            ACTUAL_CHECKSUM=$(shasum -a 256 "${TALOSCTL_PATH}" | awk '{print $1}')
        else
            echo "⚠️  No checksum tool available, skipping verification"
            exit 0
        fi

        if [ "$EXPECTED_CHECKSUM" = "$ACTUAL_CHECKSUM" ]; then
            echo "✅ Checksum verified: ${EXPECTED_CHECKSUM}"
        else
            echo "⚠️  Checksum mismatch!"
            echo "   Expected: ${EXPECTED_CHECKSUM}"
            echo "   Actual:   ${ACTUAL_CHECKSUM}"
            rm -f "${TALOSCTL_PATH}"
            exit 1
        fi

        # Print version
        "${TALOSCTL_PATH}" version --client
        exit 0
    fi
    echo "⚠️  Download attempt $i failed, retrying..."
    sleep 2
done

echo "❌ Failed to download talosctl after ${MAX_RETRIES} attempts"
exit 1
