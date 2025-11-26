#!/bin/bash
set -euo pipefail

# ===========================================================================
# Spark - EAS Build Setup Script
# ===========================================================================
# Configures Expo Application Services (EAS) for CI/CD builds
# ===========================================================================

echo "🔧 Setting up EAS Build for Spark project..."

WORKSPACE_DIR="${1:-$PWD}"
cd "$WORKSPACE_DIR"

# Check if Expo project exists
if [ ! -f "app.json" ] && [ ! -f "app.config.js" ] && [ ! -f "app.config.ts" ]; then
    echo "❌ Error: Not an Expo project. Run init-expo-project.sh first."
    exit 1
fi

# Check if EAS CLI is available
if ! command -v eas &> /dev/null; then
    echo "📦 Installing EAS CLI..."
    npm install -g eas-cli
fi

echo "📝 Creating eas.json configuration..."

# Create EAS configuration
cat > eas.json <<'EOF'
{
  "cli": {
    "version": ">= 12.0.0"
  },
  "build": {
    "development": {
      "developmentClient": true,
      "distribution": "internal",
      "ios": {
        "simulator": true
      },
      "android": {
        "buildType": "apk"
      }
    },
    "preview": {
      "distribution": "internal",
      "ios": {
        "simulator": false
      },
      "android": {
        "buildType": "apk"
      }
    },
    "production": {
      "autoIncrement": true,
      "ios": {
        "resourceClass": "m-medium"
      },
      "android": {
        "buildType": "app-bundle"
      }
    }
  },
  "submit": {
    "production": {}
  }
}
EOF

echo ""
echo "✅ EAS Build configuration complete!"
echo ""
echo "📋 Next steps:"
echo "   1. Run 'eas login' to authenticate"
echo "   2. Run 'eas build:configure' to link your project"
echo ""
