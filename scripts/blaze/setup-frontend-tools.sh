#!/bin/bash
# Setup frontend tools for Blaze agent
# Installs Playwright, v0 SDK, and other frontend dependencies

set -euo pipefail

echo "🎨 Setting up Blaze frontend tools..."

# Create temporary package.json for tool installation
cat > /tmp/blaze-tools-package.json <<'EOF'
{
  "name": "blaze-tools",
  "private": true,
  "dependencies": {
    "@playwright/test": "^1.48.0",
    "playwright": "^1.48.0",
    "@axe-core/playwright": "^4.10.0",
    "v0-sdk": "latest"
  },
  "devDependencies": {
    "@types/node": "^20.0.0",
    "typescript": "^5.0.0"
  }
}
EOF

# Install tools globally using pnpm
echo "📦 Installing Playwright, v0 SDK, and E2E testing tools..."
cd /tmp
pnpm install --global \
  @playwright/test \
  playwright \
  @axe-core/playwright \
  typescript \
  @types/node

# Install Playwright browsers (Chromium only to save space/time)
echo "🌐 Installing Playwright Chromium browser..."
pnpm exec playwright install chromium --with-deps || {
  echo "⚠️  Playwright browser install failed, will retry in entrypoint"
}

# Verify installations
echo "✅ Verifying installations..."
pnpm exec playwright --version || echo "⚠️  Playwright CLI not found"
node -e "console.log('Node.js:', process.version)"
pnpm --version

echo "✅ Blaze frontend tools setup complete!"

