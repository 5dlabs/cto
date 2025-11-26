#!/bin/bash
set -euo pipefail

# ===========================================================================
# Spark - Detox E2E Testing Script
# ===========================================================================
# Runs end-to-end tests with Detox for React Native apps
# Usage: ./run-detox-tests.sh [ios|android] [configuration]
# ===========================================================================

PLATFORM="${1:-ios}"
CONFIG="${2:-debug}"
WORKSPACE_DIR="${3:-$PWD}"

cd "$WORKSPACE_DIR"

echo "🧪 Running Detox E2E tests for Spark..."
echo "Platform: $PLATFORM"
echo "Configuration: $CONFIG"

# Check if Detox is installed
if ! grep -q "detox" package.json 2>/dev/null; then
    echo "📦 Installing Detox..."
    npm install --save-dev detox @config-plugins/detox
    npm install --save-dev jest @types/jest
    
    # Initialize Detox
    npx detox init
fi

DETOX_CONFIG=""
if [ "$PLATFORM" = "ios" ]; then
    DETOX_CONFIG="ios.sim.$CONFIG"
elif [ "$PLATFORM" = "android" ]; then
    DETOX_CONFIG="android.emu.$CONFIG"
else
    echo "❌ Invalid platform: $PLATFORM. Use 'ios' or 'android'"
    exit 1
fi

# Build and run tests
echo "🔨 Building app for $PLATFORM..."
npx detox build --configuration "$DETOX_CONFIG" || {
    echo "⚠️ Build failed. For Expo managed workflow, you may need to prebuild first:"
    echo "   npx expo prebuild"
    exit 1
}

echo "🧪 Running Detox tests..."
npx detox test --configuration "$DETOX_CONFIG" --cleanup

echo "✅ Detox tests complete!"
