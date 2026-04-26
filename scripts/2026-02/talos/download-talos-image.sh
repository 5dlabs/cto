#!/bin/bash
# Download Talos metal raw image for bare metal installation
# Usage: ./download-talos-image.sh [--version VERSION] [--arch ARCH] [--dest DIR]

set -e

TALOS_VERSION="${TALOS_VERSION:-v1.10.4}"
ARCH="${ARCH:-$(uname -m)}"
DEST="${DEST:-/var/lib/talos/images}"
CACHE_DIR="${CACHE_DIR:-$HOME/.cache/talos}"

# Detect architecture
case "$ARCH" in
    x86_64) ARCH="amd64" ;;
    aarch64|arm64) ARCH="arm64" ;;
    *)
        echo "❌ Unsupported architecture: $ARCH"
        exit 1
        ;;
esac

IMAGE_NAME="metal-${ARCH}.tar.gz"
IMAGE_URL="https://github.com/siderolabs/talos/releases/download/${TALOS_VERSION}/${IMAGE_NAME}"
CACHE_PATH="${CACHE_DIR}/${IMAGE_NAME}"
DEST_PATH="${DEST}/${IMAGE_NAME}"

echo "⬇️  Downloading Talos metal image ${TALOS_VERSION} for ${ARCH}..."
echo "   URL: ${IMAGE_URL}"
echo "   Cache: ${CACHE_PATH}"
echo "   Dest: ${DEST_PATH}"

# Create directories
mkdir -p "${DEST}"
mkdir -p "${CACHE_DIR}"

# Check cache first
if [ -f "${CACHE_PATH}" ]; then
    echo "📦 Found cached image, verifying..."

    # Verify checksum
    EXPECTED_CHECKSUM=$(curl -sL "${IMAGE_URL}.sha256" 2>/dev/null || echo "")
    ACTUAL_CHECKSUM=$(sha256sum "${CACHE_PATH}" 2>/dev/null | awk '{print $1}')

    if [ -n "$EXPECTED_CHECKSUM" ] && [ "$EXPECTED_CHECKSUM" = "$ACTUAL_CHECKSUM" ]; then
        echo "✅ Cached image verified"
        cp "${CACHE_PATH}" "${DEST_PATH}"
        echo "✅ Image copied to ${DEST_PATH}"
        ls -lh "${DEST_PATH}"
        exit 0
    else
        echo "⚠️  Cache invalid, re-downloading..."
    fi
fi

# Download with retry (follow redirects for GitHub releases)
MAX_RETRIES=3
for i in $(seq 1 $MAX_RETRIES); do
    if curl -L -o "${CACHE_PATH}" "${IMAGE_URL}" 2>/dev/null; then
        echo "✅ Image downloaded to cache: ${CACHE_PATH}"

        # Verify checksum
        echo "🔍 Verifying checksum..."
        CHECKSUM_URL="https://github.com/siderolabs/talos/releases/download/${TALOS_VERSION}/sha256sum.txt"
        EXPECTED_CHECKSUM=$(curl -sL "${CHECKSUM_URL}" | grep "${IMAGE_NAME}$" | awk '{print $1}')
        if command -v sha256sum &>/dev/null; then
            ACTUAL_CHECKSUM=$(sha256sum "${CACHE_PATH}" | awk '{print $1}')
        elif command -v shasum &>/dev/null; then
            ACTUAL_CHECKSUM=$(shasum -a 256 "${CACHE_PATH}" | awk '{print $1}')
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
            rm -f "${CACHE_PATH}"
            exit 1
        fi

        # Copy to destination
        cp "${CACHE_PATH}" "${DEST_PATH}"
        echo "✅ Image installed to ${DEST_PATH}"
        ls -lh "${DEST_PATH}"
        exit 0
    fi
    echo "⚠️  Download attempt $i failed, retrying..."
    sleep 2
done

echo "❌ Failed to download Talos image after ${MAX_RETRIES} attempts"
exit 1
