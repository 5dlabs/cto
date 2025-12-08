#!/bin/bash
# =========================================================================
# Build Dexter Agent Docker Image
#
# Usage:
#   ./scripts/build-dexter-image.sh [version]
#
# Examples:
#   ./scripts/build-dexter-image.sh          # Build with latest Dexter
#   ./scripts/build-dexter-image.sh 1.0.1    # Build with specific version
# =========================================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Version defaults to 'latest'
DEXTER_VERSION="${1:-latest}"
IMAGE_TAG="${DEXTER_VERSION}"

echo "🔨 Building Dexter agent image..."
echo "   Version: $DEXTER_VERSION"
echo "   Tag: ghcr.io/5dlabs/dexter:$IMAGE_TAG"
echo ""

# Use local Dockerfile for arm64 or when runtime image isn't available
ARCH=$(uname -m)
if [ "$ARCH" = "arm64" ] || [ "$ARCH" = "aarch64" ]; then
  echo "   Using local Dockerfile for $ARCH architecture"
  DOCKERFILE="$REPO_ROOT/infra/images/dexter/Dockerfile.local"
else
  DOCKERFILE="$REPO_ROOT/infra/images/dexter/Dockerfile"
fi

# Build the image
docker build \
  --build-arg DEXTER_VERSION="$DEXTER_VERSION" \
  -t "ghcr.io/5dlabs/dexter:$IMAGE_TAG" \
  -f "$DOCKERFILE" \
  "$REPO_ROOT/infra/images/dexter"

# Also tag as 'local' for easy testing
if [ "$IMAGE_TAG" != "local" ]; then
  docker tag "ghcr.io/5dlabs/dexter:$IMAGE_TAG" "ghcr.io/5dlabs/dexter:local"
fi

echo ""
echo "✅ Dexter image built successfully!"
echo ""
echo "Available tags:"
echo "   ghcr.io/5dlabs/dexter:$IMAGE_TAG"
echo "   ghcr.io/5dlabs/dexter:local"
echo ""
echo "Run with:"
echo "   docker run -it -e OPENAI_API_KEY=\$OPENAI_API_KEY ghcr.io/5dlabs/dexter:local dexter-agent"

