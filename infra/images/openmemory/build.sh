#!/usr/bin/env bash
set -euo pipefail

# OpenMemory Image Build Script
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
IMAGE_NAME="ghcr.io/5dlabs/openmemory"
VERSION="${1:-latest}"

echo "🐳 Building OpenMemory image..."
echo "   Registry: ${IMAGE_NAME}"
echo "   Version: ${VERSION}"

# Build the image
docker build \
  --platform linux/amd64 \
  --tag "${IMAGE_NAME}:${VERSION}" \
  --tag "${IMAGE_NAME}:latest" \
  --file "${SCRIPT_DIR}/Dockerfile" \
  "${SCRIPT_DIR}"

echo "✅ Build complete!"

# Push if requested
if [[ "${PUSH:-false}" == "true" ]]; then
  echo "📤 Pushing to registry..."
  docker push "${IMAGE_NAME}:${VERSION}"
  docker push "${IMAGE_NAME}:latest"
  echo "✅ Push complete!"
else
  echo "ℹ️  To push: PUSH=true $0 ${VERSION}"
fi
