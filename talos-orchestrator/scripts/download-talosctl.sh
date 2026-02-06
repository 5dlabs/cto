#!/bin/bash
# Download talosctl for the target architecture

set -e

ARCH="${1:-arm64}"
VERSION="${2:-1.8.0}"
OUTPUT="${3:-./talosctl}"

TALOS_URL="https://github.com/siderolabs/talos/releases/download/v${VERSION}/talosctl-darwin-${ARCH}"

echo "Downloading talosctl ${VERSION} for ${ARCH}..."
curl -fsSL "${TALOS_URL}" -o "${OUTPUT}"
chmod +x "${OUTPUT}"

echo "Downloaded talosctl to ${OUTPUT}"
