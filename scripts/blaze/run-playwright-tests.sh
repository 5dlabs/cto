#!/bin/bash
set -euo pipefail

# =========================================================================
# Blaze Playwright E2E Testing & Screenshot Capture
# =========================================================================
# Automated browser tests for Next.js applications with screenshot capture
# Usage: ./run-playwright-tests.sh <preview_url> <task_id>
# =========================================================================

if [ $# -lt 2 ]; then
    echo "Usage: $0 <preview_url> <task_id>"
    exit 1
fi

PREVIEW_URL="$1"
TASK_ID="$2"
WORK_DIR="${3:-/workspace/task-$TASK_ID}"

echo "🎭 Setting up Playwright for E2E testing"
echo "Preview URL: $PREVIEW_URL"
echo "Working directory: $WORK_DIR"

# Navigate to the project directory
cd "$WORK_DIR"

# Check if project exists
if [ ! -f "package.json" ]; then
    echo "❌ No package.json found in $WORK_DIR"
    exit 1
fi

# Install Playwright if not already installed
echo "📦 Installing Playwright..."
if ! pnpm add -D @playwright/test 2>/dev/null; then
    echo "⚠️  pnpm not available, trying npm..."
    npm install --save-dev @playwright/test
fi

# Install Playwright browsers
echo "🌐 Installing Playwright browsers..."
npx playwright install chromium --with-deps

# Create screenshots directory
SCREENSHOTS_DIR="$WORK_DIR/screenshots"
mkdir -p "$SCREENSHOTS_DIR"

# Create a basic Playwright config if it doesn't exist
if [ ! -f "playwright.config.ts" ]; then
    echo "📝 Creating Playwright configuration..."
    cat > "playwright.config.ts" <<'EOF'
import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
  testDir: './tests',
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 1 : undefined,
  reporter: 'html',
  use: {
    baseURL: process.env.PREVIEW_URL,
    trace: 'on-first-retry',
    screenshot: 'on',
  },
  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
  ],
});
EOF
fi

# Create tests directory
mkdir -p tests

# Create a basic smoke test if no tests exist
if [ ! -f "tests/smoke.spec.ts" ] && [ -z "$(ls -A tests/*.spec.ts 2>/dev/null)" ]; then
    echo "📝 Creating basic smoke test..."
    cat > "tests/smoke.spec.ts" <<'EOF'
import { test, expect } from '@playwright/test';

test('homepage loads and responds', async ({ page }) => {
  await page.goto('/');
  
  // Wait for the page to be fully loaded
  await page.waitForLoadState('networkidle');
  
  // Take a screenshot of the homepage
  await page.screenshot({ 
    path: 'screenshots/homepage.png',
    fullPage: true 
  });
  
  // Basic assertions
  await expect(page).toHaveTitle(/./);
  
  // Check if page has any visible content
  const bodyText = await page.textContent('body');
  expect(bodyText).toBeTruthy();
  expect(bodyText!.length).toBeGreaterThan(0);
  
  console.log('✅ Homepage loaded successfully');
});

test('interactive elements work', async ({ page }) => {
  await page.goto('/');
  await page.waitForLoadState('networkidle');
  
  // Try to find and interact with buttons
  const buttons = page.locator('button');
  const buttonCount = await buttons.count();
  
  if (buttonCount > 0) {
    console.log(`Found ${buttonCount} button(s) on the page`);
    
    // Take screenshot of buttons
    await page.screenshot({ 
      path: 'screenshots/interactive-elements.png',
      fullPage: true 
    });
    
    // Test first button (if it exists)
    const firstButton = buttons.first();
    const isVisible = await firstButton.isVisible();
    expect(isVisible).toBe(true);
  } else {
    console.log('⚠️  No interactive buttons found on page');
  }
});

test('navigation and routing', async ({ page }) => {
  await page.goto('/');
  await page.waitForLoadState('networkidle');
  
  // Look for links
  const links = page.locator('a[href]');
  const linkCount = await links.count();
  
  console.log(`Found ${linkCount} link(s) on the page`);
  
  if (linkCount > 0) {
    // Take screenshot showing navigation
    await page.screenshot({ 
      path: 'screenshots/navigation.png',
      fullPage: true 
    });
  }
});

test('responsive design check', async ({ page }) => {
  // Test mobile viewport
  await page.setViewportSize({ width: 375, height: 667 });
  await page.goto('/');
  await page.waitForLoadState('networkidle');
  
  await page.screenshot({ 
    path: 'screenshots/mobile-view.png',
    fullPage: true 
  });
  
  // Test tablet viewport
  await page.setViewportSize({ width: 768, height: 1024 });
  await page.goto('/');
  await page.waitForLoadState('networkidle');
  
  await page.screenshot({ 
    path: 'screenshots/tablet-view.png',
    fullPage: true 
  });
  
  // Test desktop viewport
  await page.setViewportSize({ width: 1920, height: 1080 });
  await page.goto('/');
  await page.waitForLoadState('networkidle');
  
  await page.screenshot({ 
    path: 'screenshots/desktop-view.png',
    fullPage: true 
  });
  
  console.log('✅ Responsive design screenshots captured');
});
EOF
fi

# Run Playwright tests
echo "🧪 Running Playwright E2E tests..."
export PREVIEW_URL
if npx playwright test --reporter=list; then
    echo "✅ All Playwright tests passed"
    TEST_RESULT="passed"
else
    echo "⚠️  Some Playwright tests failed (continuing...)"
    TEST_RESULT="failed"
fi

# List captured screenshots
echo ""
echo "📸 Screenshot Summary:"
if [ -d "$SCREENSHOTS_DIR" ] && [ -n "$(ls -A "$SCREENSHOTS_DIR"/*.png 2>/dev/null)" ]; then
    ls -lh "$SCREENSHOTS_DIR"/*.png | while read -r line; do
        echo "   $line"
    done
    SCREENSHOT_COUNT=$(ls -1 "$SCREENSHOTS_DIR"/*.png 2>/dev/null | wc -l)
    echo "   Total: $SCREENSHOT_COUNT screenshot(s)"
else
    echo "   ⚠️  No screenshots captured"
fi

# Generate test report summary
echo ""
echo "════════════════════════════════════════════════════════════════"
echo "║              PLAYWRIGHT TEST SUMMARY                         ║"
echo "════════════════════════════════════════════════════════════════"
echo "Preview URL: $PREVIEW_URL"
echo "Test Result: $TEST_RESULT"
echo "Screenshots: $SCREENSHOTS_DIR"
echo "════════════════════════════════════════════════════════════════"

exit 0  # Always exit 0 so workflow continues even if tests fail
