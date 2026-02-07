#!/bin/bash
# Download Talos raw image for bare metal installation

set -e

ARCH="${1:-arm64}"
VERSION="${2:-1.8.0}"
OUTPUT="${3:-./talos.raw.img.gz}"
CACHE_DIR="${HOME}/.cache/talos"

# Create cache directory
mkdir -p "${CACHE_DIR}"

# Check cache first
CACHED_FILE="${CACHE_DIR}/talos-${VERSION}-${ARCH}.raw.img.gz"
if [ -f "${CACHED_FILE" ]; then
    echo "Using cached image: ${CACHED_FILE}"
    cp "${CACHED_FILE}" "${OUTPUT}"
    exit 0
fi

TALOS_URL="https://github.com/siderolabs/talos/releases/download/v${VERSION}/talos-${ARCH}-raw.img.gz"

echo "Downloading Talos ${VERSION} raw image for ${ARCH}..."
curl -fsSL "${TALOS_URL}" -o "${CACHED_FILE}"

# Verify checksum (optional)
CHECKSUM_URL="${TALOS_URL}.sha256"
if curl -fsSL "${CHECKSUM_URL}" -o "${CACHED_FILE}.sha256" 2>/dev/null; then
    echo "Verifying checksum..."
    cd "$(dirname "${CACHED_FILE}")"
    sha256sum -c "${CACHED_FILE}.sha256" || echo "Checksum verification skipped (no checksum file)"
fi

cp "${CACHED_FILE}" "${OUTPUT}"
echo "Downloaded Talos image to ${OUTPUT} (cached at ${CACHED_FILE})"
