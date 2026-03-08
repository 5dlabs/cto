#!/bin/bash
# Build script for CTO release packages
#
# This script:
# 1. Downloads platform-specific binaries (kind, kubectl, helm, cloudflared)
# 2. Builds mcp-lite
# 3. Copies everything to resources/
# 4. Optionally runs cargo tauri build

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
RESOURCES_DIR="$ROOT_DIR/crates/cto-lite/tauri/resources"

# Versions
KIND_VERSION="v0.24.0"
KUBECTL_VERSION="v1.31.0"
HELM_VERSION="v3.16.0"
CLOUDFLARED_VERSION="2024.8.3"

# Detect platform
case "$(uname -s)" in
    Darwin)
        PLATFORM="darwin"
        ;;
    Linux)
        PLATFORM="linux"
        ;;
    MINGW*|CYGWIN*|MSYS*)
        PLATFORM="windows"
        ;;
    *)
        echo "Unsupported platform: $(uname -s)"
        exit 1
        ;;
esac

# Detect architecture
case "$(uname -m)" in
    x86_64|amd64)
        ARCH="amd64"
        KUBECTL_ARCH="amd64"
        ;;
    arm64|aarch64)
        ARCH="arm64"
        KUBECTL_ARCH="arm64"
        ;;
    *)
        echo "Unsupported architecture: $(uname -m)"
        exit 1
        ;;
esac

echo "=== CTO Build Script ==="
echo "Platform: $PLATFORM"
echo "Architecture: $ARCH"
echo "Resources directory: $RESOURCES_DIR"
echo ""

# Create directories
mkdir -p "$RESOURCES_DIR/bin"
mkdir -p "$RESOURCES_DIR/charts"
mkdir -p "$RESOURCES_DIR/templates"

# Download kind
echo "Downloading kind $KIND_VERSION..."
KIND_URL="https://kind.sigs.k8s.io/dl/$KIND_VERSION/kind-$PLATFORM-$ARCH"
curl -fsSL -o "$RESOURCES_DIR/bin/kind" "$KIND_URL"
chmod +x "$RESOURCES_DIR/bin/kind"
echo "✓ kind downloaded"

# Download kubectl
echo "Downloading kubectl $KUBECTL_VERSION..."
KUBECTL_URL="https://dl.k8s.io/release/$KUBECTL_VERSION/bin/$PLATFORM/$KUBECTL_ARCH/kubectl"
curl -fsSL -o "$RESOURCES_DIR/bin/kubectl" "$KUBECTL_URL"
chmod +x "$RESOURCES_DIR/bin/kubectl"
echo "✓ kubectl downloaded"

# Download helm
echo "Downloading helm $HELM_VERSION..."
HELM_FILENAME="helm-$HELM_VERSION-$PLATFORM-$ARCH.tar.gz"
HELM_URL="https://get.helm.sh/$HELM_FILENAME"
HELM_TMP=$(mktemp -d)
curl -fsSL -o "$HELM_TMP/$HELM_FILENAME" "$HELM_URL"
tar -xzf "$HELM_TMP/$HELM_FILENAME" -C "$HELM_TMP"
cp "$HELM_TMP/$PLATFORM-$ARCH/helm" "$RESOURCES_DIR/bin/helm"
chmod +x "$RESOURCES_DIR/bin/helm"
rm -rf "$HELM_TMP"
echo "✓ helm downloaded"

# Download cloudflared
echo "Downloading cloudflared $CLOUDFLARED_VERSION..."
if [ "$PLATFORM" = "darwin" ]; then
    if [ "$ARCH" = "arm64" ]; then
        CLOUDFLARED_URL="https://github.com/cloudflare/cloudflared/releases/download/$CLOUDFLARED_VERSION/cloudflared-darwin-arm64.tgz"
    else
        CLOUDFLARED_URL="https://github.com/cloudflare/cloudflared/releases/download/$CLOUDFLARED_VERSION/cloudflared-darwin-amd64.tgz"
    fi
    CLOUDFLARED_TMP=$(mktemp -d)
    curl -fsSL -o "$CLOUDFLARED_TMP/cloudflared.tgz" "$CLOUDFLARED_URL"
    tar -xzf "$CLOUDFLARED_TMP/cloudflared.tgz" -C "$CLOUDFLARED_TMP"
    cp "$CLOUDFLARED_TMP/cloudflared" "$RESOURCES_DIR/bin/cloudflared"
    rm -rf "$CLOUDFLARED_TMP"
elif [ "$PLATFORM" = "linux" ]; then
    CLOUDFLARED_URL="https://github.com/cloudflare/cloudflared/releases/download/$CLOUDFLARED_VERSION/cloudflared-linux-$ARCH"
    curl -fsSL -o "$RESOURCES_DIR/bin/cloudflared" "$CLOUDFLARED_URL"
fi
chmod +x "$RESOURCES_DIR/bin/cloudflared"
echo "✓ cloudflared downloaded"

# Build mcp-lite
echo "Building mcp-lite..."
cd "$ROOT_DIR"
cargo build --release -p mcp-lite
cp "$ROOT_DIR/crates/cto-lite/mcp-lite/target/release/mcp-lite" "$RESOURCES_DIR/bin/mcp-lite"
chmod +x "$RESOURCES_DIR/bin/mcp-lite"
echo "✓ mcp-lite built"

# Copy charts
echo "Copying Helm charts..."
rm -rf "$RESOURCES_DIR/charts/cto-lite"
cp -r "$ROOT_DIR/infra/charts/cto-lite" "$RESOURCES_DIR/charts/"
echo "✓ Helm charts copied"

# Copy templates
echo "Copying workflow templates..."
cp "$ROOT_DIR/templates/workflows/play-workflow-lite.yaml" "$RESOURCES_DIR/templates/"
echo "✓ Templates copied"

# Summary
echo ""
echo "=== Build Resources Ready ==="
echo "Contents of $RESOURCES_DIR:"
find "$RESOURCES_DIR" -type f | while read f; do
    echo "  $(ls -lh "$f" | awk '{print $5, $9}')"
done

# Optionally build Tauri app
if [ "$1" = "--build" ]; then
    echo ""
    echo "Building Tauri application..."
    cd "$ROOT_DIR/crates/cto-lite/tauri"
    cargo tauri build
    echo "✓ Tauri build complete"
fi

echo ""
echo "Done! Resources are ready in: $RESOURCES_DIR"
echo ""
echo "To build the Tauri app:"
echo "  cd crates/cto-lite/tauri && cargo tauri build"
