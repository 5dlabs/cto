import { test, expect } from '@playwright/test';
import * as fs from 'fs';
import * as path from 'path';

/**
 * Screenshot tests for different viewport sizes
 * Captures mobile, tablet, and desktop views
 */

test.describe('Screenshot Tests', () => {
  const screenshotDir = path.join(__dirname, '../screenshots');

  test.beforeAll(() => {
    // Ensure screenshot directory exists
    if (!fs.existsSync(screenshotDir)) {
      fs.mkdirSync(screenshotDir, { recursive: true });
    }
  });

  test('capture mobile screenshot (375x667)', async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 667 });
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    await page.screenshot({
      path: path.join(screenshotDir, 'mobile-375px.png'),
      fullPage: true,
    });

    // Verify key elements are visible
    await expect(page.locator('body')).toBeVisible();
  });

  test('capture tablet screenshot (768x1024)', async ({ page }) => {
    await page.setViewportSize({ width: 768, height: 1024 });
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    await page.screenshot({
      path: path.join(screenshotDir, 'tablet-768px.png'),
      fullPage: true,
    });

    await expect(page.locator('body')).toBeVisible();
  });

  test('capture desktop screenshot (1920x1080)', async ({ page }) => {
    await page.setViewportSize({ width: 1920, height: 1080 });
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    await page.screenshot({
      path: path.join(screenshotDir, 'desktop-1920px.png'),
      fullPage: true,
    });

    await expect(page.locator('body')).toBeVisible();
  });

  test('capture component-specific screenshots', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');

    // Find elements with data-component attribute
    const components = await page.locator('[data-component]').all();

    for (const component of components) {
      const componentName = await component.getAttribute('data-component');
      if (componentName) {
        await component.screenshot({
          path: path.join(screenshotDir, `component-${componentName}.png`),
        });
      }
    }
  });

  test('capture interaction states', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');

    // Capture hover states on buttons
    const buttons = await page.locator('button').all();
    for (let i = 0; i < Math.min(buttons.length, 5); i++) {
      const button = buttons[i];
      await button.hover();
      await page.screenshot({
        path: path.join(screenshotDir, `button-hover-${i}.png`),
      });
    }

    // Capture focus states
    const focusableElements = await page.locator('button, a, input').all();
    for (let i = 0; i < Math.min(focusableElements.length, 5); i++) {
      const element = focusableElements[i];
      await element.focus();
      await page.screenshot({
        path: path.join(screenshotDir, `focus-state-${i}.png`),
      });
    }
  });

  test('capture error states', async ({ page }) => {
    await page.goto('/');
    
    // Try to trigger form validation errors
    const forms = await page.locator('form').all();
    for (const form of forms) {
      // Try submitting without filling required fields
      const submitButton = form.locator('button[type="submit"]');
      if (await submitButton.count() > 0) {
        await submitButton.click();
        await page.waitForTimeout(500); // Wait for validation
        await page.screenshot({
          path: path.join(screenshotDir, 'form-validation-errors.png'),
        });
      }
    }
  });
});

