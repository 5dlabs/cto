#!/bin/bash
set -euo pipefail

# ===========================================================================
# Spark - Bare React Native Project Initialization Script
# ===========================================================================
# This script initializes a bare React Native project (not Expo managed)
# Use this when you need full native control or unsupported native modules
# ===========================================================================

echo "📱 Spark: Initializing bare React Native project..."

WORKSPACE_DIR="${1:-$PWD}"
PROJECT_NAME="${2:-MobileApp}"

cd "$WORKSPACE_DIR"

echo "📍 Working directory: $WORKSPACE_DIR"
echo "📦 Project name: $PROJECT_NAME"

# Check if React Native CLI is available
if ! command -v npx &> /dev/null; then
    echo "❌ Error: npx is not available"
    exit 1
fi

# Check if project already exists
if [ -f "package.json" ] && grep -q "react-native" package.json 2>/dev/null; then
    echo "✅ React Native project already exists, skipping initialization"
    exit 0
fi

echo "🚀 Creating bare React Native project..."

# Initialize React Native project with TypeScript template
npx @react-native-community/cli@latest init "$PROJECT_NAME" \
    --template react-native-template-typescript \
    --skip-install

# Move contents if created in subdirectory
if [ -d "$PROJECT_NAME" ]; then
    mv "$PROJECT_NAME"/* "$PROJECT_NAME"/.* . 2>/dev/null || true
    rmdir "$PROJECT_NAME" 2>/dev/null || true
fi

echo "📦 Installing dependencies..."
npm install

echo "📦 Installing NativeWind..."
npm install nativewind
npm install --save-dev tailwindcss

echo "📦 Installing common React Native dependencies..."
npm install @react-navigation/native @react-navigation/native-stack
npm install react-native-screens react-native-safe-area-context
npm install react-native-gesture-handler
npm install @react-native-async-storage/async-storage
npm install react-native-reanimated

echo ""
echo "✅ Bare React Native project initialized successfully!"
echo ""
echo "🛠️ Available commands:"
echo "   npx react-native start        # Start Metro bundler"
echo "   npx react-native run-ios      # Run on iOS simulator"
echo "   npx react-native run-android  # Run on Android emulator"
echo ""
echo "📚 Next steps:"
echo "   1. cd ios && pod install      # Install iOS dependencies"
echo "   2. Add more screens in src/screens/"
echo ""
