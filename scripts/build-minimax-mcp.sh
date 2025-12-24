#!/usr/bin/env bash
set -euo pipefail

# Build script for minimax-mcp MCP server package
# Compiles TypeScript to JavaScript for npm publishing

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
PACKAGE_DIR="$REPO_ROOT/packages/minimax-mcp"

cd "$PACKAGE_DIR"

echo "🔨 Building minimax-mcp package..."

# Check for Node.js
if ! command -v node &> /dev/null; then
    echo "❌ Node.js not found. Please install Node.js >= 18.0.0"
    exit 1
fi

# Check Node.js version
NODE_VERSION=$(node -v | sed 's/v//')
REQUIRED_VERSION="18.0.0"

if [ "$(printf '%s\n' "$REQUIRED_VERSION" "$NODE_VERSION" | sort -V | head -n1)" != "$REQUIRED_VERSION" ]; then
    echo "❌ Node.js version $NODE_VERSION is below required version $REQUIRED_VERSION"
    exit 1
fi

# Install dependencies if node_modules doesn't exist
if [ ! -d "node_modules" ]; then
    echo "📦 Installing dependencies..."
    npm install
fi

# Clean previous build
if [ -d "dist" ]; then
    echo "🧹 Cleaning previous build..."
    rm -rf dist
fi

# Build TypeScript
echo "📝 Compiling TypeScript..."
npm run build

# Verify build output
if [ ! -f "dist/index.js" ]; then
    echo "❌ Build failed: dist/index.js not found"
    exit 1
fi

# Make entry point executable (it has #!/usr/bin/env node shebang)
chmod +x dist/index.js

echo "✅ Build complete: dist/index.js"
ls -lh dist/

