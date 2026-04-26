#!/usr/bin/env bash
# Migrate container images from GitHub Container Registry (ghcr.io) to GitLab Registry
# Run this after GitLab CE is deployed and the registry is accessible at registry.5dlabs.ai
#
# Prerequisites:
#   docker login ghcr.io -u <github-user> -p <github-pat>
#   docker login registry.5dlabs.ai -u <gitlab-user> -p <gitlab-pat>

set -euo pipefail

SOURCE_REGISTRY="ghcr.io/5dlabs"
TARGET_REGISTRY="registry.5dlabs.ai/5dlabs"

# All images to migrate
IMAGES=(
  # Core platform
  runtime
  agents
  controller
  pm-server
  tools
  healer
  openmemory
  web
  grok
  research
  tweakcn
  linear-sidecar

  # Build infrastructure
  rust-builder

  # Agent images
  agent-morgan
  agent-grizz
  agent-nova
  agent-blaze
  agent-tess
  agent-cleo
  agent-cipher
  agent-bolt

  # Other
  openclaw
  ralph-runner
  discord-bridge
  linear-bridge
)

# Tags to migrate per image
TAGS=(latest)

echo "=== Container Registry Migration ==="
echo "Source: ${SOURCE_REGISTRY}"
echo "Target: ${TARGET_REGISTRY}"
echo ""

FAILED=()
SUCCEEDED=()

for IMAGE in "${IMAGES[@]}"; do
  for TAG in "${TAGS[@]}"; do
    SRC="${SOURCE_REGISTRY}/${IMAGE}:${TAG}"
    DST="${TARGET_REGISTRY}/${IMAGE}:${TAG}"

    echo "--- Migrating: ${IMAGE}:${TAG} ---"

    # Pull from GHCR
    if ! docker pull "${SRC}" 2>/dev/null; then
      echo "  SKIP: ${SRC} not found"
      continue
    fi

    # Re-tag for GitLab
    docker tag "${SRC}" "${DST}"

    # Push to GitLab
    if docker push "${DST}"; then
      echo "  OK: ${DST}"
      SUCCEEDED+=("${IMAGE}:${TAG}")
    else
      echo "  FAIL: Could not push ${DST}"
      FAILED+=("${IMAGE}:${TAG}")
    fi

    # Clean up local tags
    docker rmi "${SRC}" "${DST}" 2>/dev/null || true

    echo ""
  done
done

echo "=== Migration Summary ==="
echo "Succeeded: ${#SUCCEEDED[@]}"
for img in "${SUCCEEDED[@]}"; do
  echo "  - ${img}"
done

if [ ${#FAILED[@]} -gt 0 ]; then
  echo ""
  echo "Failed: ${#FAILED[@]}"
  for img in "${FAILED[@]}"; do
    echo "  - ${img}"
  done
  exit 1
fi

echo ""
echo "All images migrated successfully."
