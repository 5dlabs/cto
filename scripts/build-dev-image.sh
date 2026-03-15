#!/usr/bin/env bash
# =============================================================================
# Fast Local Dev Image Builder
# =============================================================================
#
# Builds and pushes agent images with locally-compiled binaries for rapid iteration.
# Bypasses GitHub Actions for quick testing of intake/runtime changes.
#
# Prerequisites:
#   - cargo-zigbuild: cargo install cargo-zigbuild
#   - Docker with buildx
#   - Registry authentication: echo $GITHUB_TOKEN | docker login registry.5dlabs.ai -u USERNAME --password-stdin
#
# Usage:
#   ./scripts/build-dev-image.sh [--binary intake|pm-activity|all] [--image runtime|claude] [--push]
#
# Examples:
#   ./scripts/build-dev-image.sh --binary intake --image runtime --push
#   ./scripts/build-dev-image.sh --binary all --image claude --push
#   ./scripts/build-dev-image.sh --binary intake  # Build only, don't push
#
# =============================================================================

set -euo pipefail

# Defaults
BINARY="intake"
IMAGE="runtime"
PUSH=false
DEV_TAG="dev"
REGISTRY="registry.5dlabs.ai/5dlabs"

# Parse arguments
while [[ $# -gt 0 ]]; do
  case $1 in
    --binary)
      BINARY="$2"
      shift 2
      ;;
    --image)
      IMAGE="$2"
      shift 2
      ;;
    --push)
      PUSH=true
      shift
      ;;
    --tag)
      DEV_TAG="$2"
      shift 2
      ;;
    -h|--help)
      head -30 "$0" | tail -25
      exit 0
      ;;
    *)
      echo "Unknown option: $1"
      exit 1
      ;;
  esac
done

echo "╔══════════════════════════════════════════════════════════════════════════════╗"
echo "║                        CTO DEV IMAGE BUILDER                                 ║"
echo "╚══════════════════════════════════════════════════════════════════════════════╝"
echo ""
echo "📦 Binary:  $BINARY"
echo "🐳 Image:   $IMAGE"
echo "🏷️  Tag:     $DEV_TAG"
echo "📤 Push:    $PUSH"
echo ""

# Check prerequisites
if ! command -v cargo-zigbuild &> /dev/null; then
  echo "❌ cargo-zigbuild not found. Install with: cargo install cargo-zigbuild"
  echo "   Alternative: cargo install cross"
  exit 1
fi

# Ensure we're in the repo root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(dirname "$SCRIPT_DIR")"
cd "$REPO_ROOT"

# Create temp directory for build artifacts
BUILD_DIR=$(mktemp -d)
trap 'rm -rf "$BUILD_DIR"' EXIT

echo "═══ Step 1: Cross-compiling for linux-x86_64 ═══"

compile_binary() {
  local name=$1
  local package=$2
  
  echo "🔨 Building $name..."
  cargo zigbuild --release --target x86_64-unknown-linux-gnu -p "$package" --bin "$name"
  
  local src="target/x86_64-unknown-linux-gnu/release/$name"
  if [[ -f "$src" ]]; then
    cp "$src" "$BUILD_DIR/$name"
    chmod +x "$BUILD_DIR/$name"
    echo "✅ Built: $name ($(du -h "$BUILD_DIR/$name" | cut -f1))"
  else
    echo "❌ Build failed: $src not found"
    exit 1
  fi
}

case "$BINARY" in
  intake)
    compile_binary "intake" "intake"
    ;;
  pm-activity)
    compile_binary "pm-activity" "pm"
    ;;
  installer)
    compile_binary "installer" "installer"
    ;;
  all)
    compile_binary "intake" "intake"
    compile_binary "pm-activity" "pm"
    compile_binary "installer" "installer"
    ;;
  *)
    echo "❌ Unknown binary: $BINARY (options: intake, pm-activity, installer, all)"
    exit 1
    ;;
esac

echo ""
echo "═══ Step 2: Building Docker image ═══"

# Determine base image and final image name
case "$IMAGE" in
  runtime)
    BASE_IMAGE="$REGISTRY/runtime:latest"
    FINAL_IMAGE="$REGISTRY/runtime:$DEV_TAG"
    ;;
  claude)
    BASE_IMAGE="$REGISTRY/claude:latest"
    FINAL_IMAGE="$REGISTRY/claude:$DEV_TAG"
    ;;
  cursor)
    BASE_IMAGE="$REGISTRY/cursor:latest"
    FINAL_IMAGE="$REGISTRY/cursor:$DEV_TAG"
    ;;
  codex)
    BASE_IMAGE="$REGISTRY/codex:latest"
    FINAL_IMAGE="$REGISTRY/codex:$DEV_TAG"
    ;;
  *)
    echo "❌ Unknown image: $IMAGE (options: runtime, claude, cursor, codex)"
    exit 1
    ;;
esac

# Generate minimal Dockerfile
cat > "$BUILD_DIR/Dockerfile" <<EOF
# Dev overlay image - replaces binaries in existing image
FROM $BASE_IMAGE

USER root
EOF

# Add COPY instructions for each binary
for bin in "$BUILD_DIR"/*; do
  [[ "$(basename "$bin")" == "Dockerfile" ]] && continue
  bin_name=$(basename "$bin")
  echo "COPY $bin_name /usr/local/bin/$bin_name" >> "$BUILD_DIR/Dockerfile"
  echo "RUN chmod +x /usr/local/bin/$bin_name" >> "$BUILD_DIR/Dockerfile"
done

# Restore user
cat >> "$BUILD_DIR/Dockerfile" <<EOF

USER node
EOF

echo "📄 Generated Dockerfile:"
cat "$BUILD_DIR/Dockerfile"
echo ""

# Build the image
echo "🐳 Building $FINAL_IMAGE..."
docker buildx build \
  --platform linux/amd64 \
  --load \
  -t "$FINAL_IMAGE" \
  "$BUILD_DIR"

echo "✅ Image built: $FINAL_IMAGE"

if [[ "$PUSH" == "true" ]]; then
  echo ""
  echo "═══ Step 3: Pushing to registry ═══"
  
  # Check if logged in
  if ! docker manifest inspect "$BASE_IMAGE" &> /dev/null; then
    echo "⚠️  Not logged into registry. Run:"
    echo "   echo \$GITHUB_TOKEN | docker login registry.5dlabs.ai -u YOUR_USERNAME --password-stdin"
    exit 1
  fi
  
  echo "📤 Pushing $FINAL_IMAGE..."
  docker push "$FINAL_IMAGE"
  echo "✅ Pushed: $FINAL_IMAGE"
  
  echo ""
  echo "═══ Step 4: Usage ═══"
  echo ""
  echo "To use this dev image in your cluster:"
  echo ""
  echo "  # Option A: Patch existing deployment"
  echo "  kubectl set image deployment/claude-agent claude=$FINAL_IMAGE -n cto"
  echo ""
  echo "  # Option B: Update CodeRun to use dev image"
  echo "  # Add to your PRD or cto-config.json:"
  echo "  #   \"agentImage\": \"$FINAL_IMAGE\""
  echo ""
  echo "  # Option C: Create a test CodeRun"
  echo "  kubectl apply -f - <<YAML"
  echo "  apiVersion: agents.platform/v1"
  echo "  kind: CodeRun"
  echo "  metadata:"
  echo "    name: test-dev-intake"
  echo "    namespace: cto"
  echo "  spec:"
  echo "    cli: claude"
  echo "    prompt: \"Run intake --version and report the output\""
  echo "    image: $FINAL_IMAGE"
  echo "  YAML"
else
  echo ""
  echo "═══ Next Steps ═══"
  echo ""
  echo "Image built locally. To push to GHCR, run:"
  echo "  $0 --binary $BINARY --image $IMAGE --push"
  echo ""
  echo "Or push manually:"
  echo "  docker push $FINAL_IMAGE"
fi

echo ""
echo "════════════════════════════════════════════════════════════════════════════════"
echo "✅ Done!"
echo "════════════════════════════════════════════════════════════════════════════════"
