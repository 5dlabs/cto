#!/bin/bash
set -euo pipefail

# ============================================================================
# Blaze - Run Playwright E2E Tests
# ============================================================================
# Executes Playwright test suite and generates reports
# ============================================================================

PLAYWRIGHT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/playwright" && pwd)"
BASE_URL="${BASE_URL:-http://localhost:3000}"
CI="${CI:-false}"

echo "🎭 Running Playwright E2E tests..."
echo "   Test directory: $PLAYWRIGHT_DIR"
echo "   Base URL: $BASE_URL"

cd "$PLAYWRIGHT_DIR"

# Ensure Playwright is installed
if [ ! -d "node_modules/@playwright/test" ]; then
    echo "📦 Installing Playwright dependencies..."
    pnpm add -D @playwright/test @axe-core/playwright
    pnpm exec playwright install --with-deps
fi

# Set environment
export BASE_URL
export CI

# Run tests
echo "🚀 Executing test suite..."
pnpm exec playwright test \
    --reporter=html,json,list \
    --output=test-results

EXIT_CODE=$?

# Generate report summary
echo ""
if [ $EXIT_CODE -eq 0 ]; then
    echo "✅ All tests passed!"
else
    echo "❌ Some tests failed (exit code: $EXIT_CODE)"
fi

# Output results location
echo ""
echo "📊 Test Results:"
echo "   HTML Report: $PLAYWRIGHT_DIR/playwright-report/index.html"
echo "   JSON Results: $PLAYWRIGHT_DIR/test-results.json"
echo "   Screenshots: $PLAYWRIGHT_DIR/screenshots/"
echo ""

# If in CI, fail the script if tests failed
if [ "$CI" = "true" ] && [ $EXIT_CODE -ne 0 ]; then
    exit $EXIT_CODE
fi

# Always exit 0 in dev mode to not break workflows
exit 0

