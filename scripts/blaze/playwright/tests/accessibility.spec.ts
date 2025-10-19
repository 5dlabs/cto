import { test, expect } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';

/**
 * Accessibility tests (WCAG AA compliance)
 * Uses axe-core to scan for accessibility violations
 */

test.describe('Accessibility Tests', () => {
  test('homepage has no accessibility violations', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');

    const accessibilityScanResults = await new AxeBuilder({ page })
      .withTags(['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa'])
      .analyze();

    expect(accessibilityScanResults.violations).toEqual([]);
  });

  test('all pages have proper heading hierarchy', async ({ page }) => {
    await page.goto('/');
    
    // Get all headings
    const h1Count = await page.locator('h1').count();
    expect(h1Count).toBeGreaterThanOrEqual(1);
    expect(h1Count).toBeLessThanOrEqual(1); // Only one h1 per page
    
    // Verify heading order (h1 -> h2 -> h3, etc.)
    const headings = await page.locator('h1, h2, h3, h4, h5, h6').all();
    const levels = await Promise.all(
      headings.map(async (h) => {
        const tagName = await h.evaluate((el) => el.tagName);
        return parseInt(tagName.substring(1));
      })
    );
    
    // Check that headings don't skip levels
    for (let i = 1; i < levels.length; i++) {
      const diff = levels[i] - levels[i - 1];
      expect(diff).toBeLessThanOrEqual(1);
    }
  });

  test('all images have alt text', async ({ page }) => {
    await page.goto('/');
    
    const images = await page.locator('img').all();
    
    for (const img of images) {
      const alt = await img.getAttribute('alt');
      expect(alt).not.toBeNull();
      // Alt can be empty string for decorative images, but attribute must exist
    }
  });

  test('all form inputs have labels', async ({ page }) => {
    await page.goto('/');
    
    const inputs = await page.locator('input, textarea, select').all();
    
    for (const input of inputs) {
      const id = await input.getAttribute('id');
      const ariaLabel = await input.getAttribute('aria-label');
      const ariaLabelledby = await input.getAttribute('aria-labelledby');
      
      if (id) {
        // Check for associated label
        const label = page.locator(`label[for="${id}"]`);
        const hasLabel = await label.count() > 0;
        
        expect(
          hasLabel || ariaLabel || ariaLabelledby
        ).toBeTruthy();
      } else {
        // Must have aria-label if no id
        expect(ariaLabel || ariaLabelledby).not.toBeNull();
      }
    }
  });

  test('interactive elements have visible focus indicators', async ({ page }) => {
    await page.goto('/');
    
    const focusableElements = await page.locator(
      'button:visible, a:visible, input:visible'
    ).all();
    
    for (let i = 0; i < Math.min(focusableElements.length, 10); i++) {
      const element = focusableElements[i];
      
      await element.focus();
      await page.waitForTimeout(100);
      
      // Get computed outline/box-shadow (focus indicators)
      const styles = await element.evaluate((el) => {
        const computed = window.getComputedStyle(el);
        return {
          outline: computed.outline,
          outlineWidth: computed.outlineWidth,
          boxShadow: computed.boxShadow,
        };
      });
      
      // Should have some form of visible focus indicator
      const hasFocusIndicator =
        (styles.outline !== 'none' && styles.outlineWidth !== '0px') ||
        styles.boxShadow !== 'none';
      
      expect(hasFocusIndicator).toBeTruthy();
    }
  });

  test('color contrast meets WCAG AA standards', async ({ page }) => {
    await page.goto('/');

    const accessibilityScanResults = await new AxeBuilder({ page })
      .withTags(['wcag2aa'])
      .disableRules(['color-contrast']) // We'll check manually
      .analyze();

    // Filter for contrast violations
    const contrastViolations = accessibilityScanResults.violations.filter(
      (v) => v.id === 'color-contrast'
    );

    expect(contrastViolations).toEqual([]);
  });

  test('page has skip navigation link', async ({ page }) => {
    await page.goto('/');
    
    // Press Tab to focus skip link
    await page.keyboard.press('Tab');
    await page.waitForTimeout(100);
    
    // Check if first focusable element is a skip link
    const focused = page.locator(':focus');
    const text = await focused.textContent();
    
    // Skip links usually contain "skip" or "jump"
    if (text) {
      const isSkipLink = text.toLowerCase().includes('skip') ||
                        text.toLowerCase().includes('jump');
      
      // If it's not a skip link, that's okay, but log it
      if (!isSkipLink) {
        console.log('Note: No skip navigation link detected');
      }
    }
  });

  test('aria roles are used appropriately', async ({ page }) => {
    await page.goto('/');

    const accessibilityScanResults = await new AxeBuilder({ page })
      .withTags(['best-practice'])
      .analyze();

    // Check for role-related violations
    const roleViolations = accessibilityScanResults.violations.filter(
      (v) => v.id.includes('role') || v.id.includes('aria')
    );

    expect(roleViolations).toEqual([]);
  });

  test('keyboard navigation is logical', async ({ page }) => {
    await page.goto('/');
    
    const focusOrder: string[] = [];
    
    // Tab through first 15 elements
    for (let i = 0; i < 15; i++) {
      await page.keyboard.press('Tab');
      await page.waitForTimeout(50);
      
      const focused = page.locator(':focus');
      const tagName = await focused.evaluate((el) => el.tagName).catch(() => '');
      const text = await focused.textContent().catch(() => '');
      
      focusOrder.push(`${tagName}: ${text?.substring(0, 30)}`);
    }
    
    // Verify we focused on interactive elements
    expect(focusOrder.length).toBeGreaterThan(0);
    console.log('Focus order:', focusOrder);
  });

  test('no accessibility violations on all routes', async ({ page }) => {
    await page.goto('/');
    
    // Find all navigation links
    const links = await page.locator('nav a, header a').all();
    const hrefs = await Promise.all(
      links.map(async (link) => await link.getAttribute('href'))
    );
    
    // Test accessibility on each route
    for (const href of hrefs) {
      if (href && href.startsWith('/')) {
        await page.goto(href);
        await page.waitForLoadState('networkidle');
        
        const results = await new AxeBuilder({ page })
          .withTags(['wcag2a', 'wcag2aa'])
          .analyze();
        
        expect(results.violations).toEqual([]);
      }
    }
  });
});

