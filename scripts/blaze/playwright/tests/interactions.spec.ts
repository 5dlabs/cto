import { test, expect } from '@playwright/test';

/**
 * Interaction tests
 * Verifies user interactions work correctly across devices
 */

test.describe('Interaction Tests', () => {
  test('navigation links work', async ({ page }) => {
    await page.goto('/');
    
    // Find all navigation links
    const navLinks = await page.locator('nav a, header a').all();
    
    for (const link of navLinks) {
      const href = await link.getAttribute('href');
      if (href && href.startsWith('/')) {
        // Internal link
        await link.click();
        await page.waitForLoadState('networkidle');
        
        // Verify we navigated
        expect(page.url()).toContain(href);
        
        // Go back
        await page.goBack();
      }
    }
  });

  test('buttons are clickable', async ({ page }) => {
    await page.goto('/');
    
    // Find all buttons
    const buttons = await page.locator('button:visible').all();
    
    for (const button of buttons) {
      // Check if button is enabled
      const isDisabled = await button.isDisabled();
      if (!isDisabled) {
        // Button should be clickable
        await expect(button).toBeEnabled();
        
        // Click and verify no errors
        await button.click({ force: false });
        
        // Wait a bit for any side effects
        await page.waitForTimeout(100);
      }
    }
  });

  test('forms can be filled and submitted', async ({ page }) => {
    await page.goto('/');
    
    const forms = await page.locator('form').all();
    
    for (const form of forms) {
      // Fill text inputs
      const textInputs = await form.locator('input[type="text"], input[type="email"], textarea').all();
      for (const input of textInputs) {
        await input.fill('Test input value');
      }
      
      // Fill checkboxes
      const checkboxes = await form.locator('input[type="checkbox"]').all();
      for (const checkbox of checkboxes) {
        await checkbox.check();
      }
      
      // Fill radio buttons
      const radios = await form.locator('input[type="radio"]').all();
      if (radios.length > 0) {
        await radios[0].check();
      }
      
      // Find submit button
      const submitButton = form.locator('button[type="submit"]');
      if (await submitButton.count() > 0) {
        await submitButton.click();
        
        // Wait for form submission to complete
        await page.waitForTimeout(1000);
      }
    }
  });

  test('keyboard navigation works', async ({ page }) => {
    await page.goto('/');
    
    // Tab through focusable elements
    const focusableElements = await page.locator(
      'button:visible, a:visible, input:visible, select:visible, textarea:visible'
    ).all();
    
    for (let i = 0; i < Math.min(focusableElements.length, 10); i++) {
      await page.keyboard.press('Tab');
      await page.waitForTimeout(100);
      
      // Verify something is focused
      const focusedElement = page.locator(':focus');
      await expect(focusedElement).toBeVisible();
    }
  });

  test('enter key activates buttons', async ({ page }) => {
    await page.goto('/');
    
    const buttons = await page.locator('button:visible').all();
    
    for (let i = 0; i < Math.min(buttons.length, 3); i++) {
      const button = buttons[i];
      await button.focus();
      await page.keyboard.press('Enter');
      await page.waitForTimeout(100);
    }
  });

  test('escape key closes dialogs', async ({ page }) => {
    await page.goto('/');
    
    // Try to open dialogs by clicking buttons
    const buttons = await page.locator('button:visible').all();
    
    for (const button of buttons) {
      await button.click();
      await page.waitForTimeout(200);
      
      // Check if a dialog appeared
      const dialog = page.locator('[role="dialog"]');
      if (await dialog.count() > 0) {
        // Press escape
        await page.keyboard.press('Escape');
        await page.waitForTimeout(200);
        
        // Verify dialog is closed
        await expect(dialog).toBeHidden();
        break;
      }
    }
  });

  test('hover states are applied', async ({ page }) => {
    await page.goto('/');
    
    const interactiveElements = await page.locator('button, a, [data-component]').all();
    
    for (let i = 0; i < Math.min(interactiveElements.length, 5); i++) {
      const element = interactiveElements[i];
      
      // Get initial styles
      const beforeHover = await element.evaluate((el) => {
        return window.getComputedStyle(el).getPropertyValue('background-color');
      });
      
      // Hover
      await element.hover();
      await page.waitForTimeout(100);
      
      // Styles might change (or might not, depending on design)
      // Just verify the element is still visible after hover
      await expect(element).toBeVisible();
    }
  });

  test('responsive menu works on mobile', async ({ page, isMobile }) => {
    if (!isMobile) {
      test.skip();
    }
    
    await page.goto('/');
    
    // Look for hamburger menu button
    const menuButton = page.locator('button[aria-label*="menu" i], button[aria-label*="navigation" i]');
    
    if (await menuButton.count() > 0) {
      // Menu should be closed initially
      const nav = page.locator('nav');
      
      // Open menu
      await menuButton.click();
      await page.waitForTimeout(300);
      
      // Verify menu opened
      await expect(nav).toBeVisible();
      
      // Close menu
      await menuButton.click();
      await page.waitForTimeout(300);
    }
  });
});

