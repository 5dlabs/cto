#!/bin/bash
# Start the web app in development mode
set -euo pipefail

cd "$(dirname "$0")/../apps/web"

# Check if .env.local exists, if not create from template
if [ ! -f .env.local ]; then
  echo "⚠️  No .env.local found. Creating from template..."
  cat > .env.local << 'ENVEOF'
# Web App Environment Variables
# Copy from root env.template if needed

# Database URL (Neon/PostgreSQL) - Optional for basic development
# Format: postgresql://user@host:port/database?sslmode=require
# DATABASE_URL="postgresql://user@host:port/database?sslmode=require"

# Better Auth secret (generate with: openssl rand -base64 32)
# BETTER_AUTH_SECRET=""

# GitHub OAuth credentials - Optional for auth features
# GITHUB_CLIENT_ID=""
# GITHUB_CLIENT_SECRET=""

# Public app URL
NEXT_PUBLIC_APP_URL="http://localhost:3000"

# Disable telemetry
NEXT_TELEMETRY_DISABLED=1
ENVEOF
  echo "✅ Created .env.local with defaults"
fi

echo "🚀 Starting Next.js dev server at http://localhost:3000"
npm run dev
