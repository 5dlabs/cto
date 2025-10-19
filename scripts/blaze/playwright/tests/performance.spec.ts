import { test, expect } from '@playwright/test';

/**
 * Performance tests
 * Measures page load times, bundle sizes, and Core Web Vitals
 */

test.describe('Performance Tests', () => {
  test('page load time is under 3 seconds', async ({ page }) => {
    const startTime = Date.now();
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    const loadTime = Date.now() - startTime;

    console.log(`Page load time: ${loadTime}ms`);
    expect(loadTime).toBeLessThan(3000); // 3 seconds
  });

  test('First Contentful Paint (FCP) is under 1.8s', async ({ page }) => {
    await page.goto('/');
    
    const fcp = await page.evaluate(() => {
      const perfEntries = performance.getEntriesByType('paint');
      const fcpEntry = perfEntries.find((entry) => entry.name === 'first-contentful-paint');
      return fcpEntry ? fcpEntry.startTime : 0;
    });

    console.log(`FCP: ${fcp}ms`);
    expect(fcp).toBeLessThan(1800); // Good FCP score
  });

  test('Largest Contentful Paint (LCP) is under 2.5s', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    const lcp = await page.evaluate(() => {
      return new Promise<number>((resolve) => {
        const observer = new PerformanceObserver((list) => {
          const entries = list.getEntries();
          const lastEntry = entries[entries.length - 1];
          resolve(lastEntry.startTime);
          observer.disconnect();
        });
        observer.observe({ type: 'largest-contentful-paint', buffered: true });
        
        // Fallback timeout
        setTimeout(() => resolve(0), 5000);
      });
    });

    console.log(`LCP: ${lcp}ms`);
    expect(lcp).toBeLessThan(2500); // Good LCP score
  });

  test('Cumulative Layout Shift (CLS) is under 0.1', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    // Wait for layout to settle
    await page.waitForTimeout(2000);
    
    const cls = await page.evaluate(() => {
      return new Promise<number>((resolve) => {
        let clsValue = 0;
        const observer = new PerformanceObserver((list) => {
          for (const entry of list.getEntries()) {
            if (!(entry as any).hadRecentInput) {
              clsValue += (entry as any).value;
            }
          }
        });
        observer.observe({ type: 'layout-shift', buffered: true });
        
        setTimeout(() => {
          observer.disconnect();
          resolve(clsValue);
        }, 1000);
      });
    });

    console.log(`CLS: ${cls}`);
    expect(cls).toBeLessThan(0.1); // Good CLS score
  });

  test('Time to Interactive (TTI) is reasonable', async ({ page }) => {
    const startTime = Date.now();
    await page.goto('/');
    await page.waitForLoadState('domcontentloaded');
    
    // Wait for interactive elements to be clickable
    await page.locator('button, a').first().waitFor({ state: 'visible' });
    
    const tti = Date.now() - startTime;
    
    console.log(`TTI: ${tti}ms`);
    expect(tti).toBeLessThan(3500); // Should be interactive quickly
  });

  test('bundle size is reasonable', async ({ page }) => {
    // Intercept network requests
    const jsRequests: any[] = [];
    
    page.on('response', (response) => {
      const url = response.url();
      if (url.endsWith('.js') && response.ok()) {
        jsRequests.push({
          url,
          size: response.headers()['content-length'],
        });
      }
    });
    
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    // Calculate total JS size
    const totalJsSize = jsRequests.reduce((sum, req) => {
      const size = parseInt(req.size || '0', 10);
      return sum + size;
    }, 0);
    
    console.log(`Total JS size: ${(totalJsSize / 1024).toFixed(2)} KB`);
    
    // Modern Next.js apps should keep initial JS under 300KB
    expect(totalJsSize).toBeLessThan(300 * 1024);
  });

  test('images are optimized', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    const images = await page.locator('img').all();
    
    for (const img of images) {
      const src = await img.getAttribute('src');
      const loading = await img.getAttribute('loading');
      const width = await img.getAttribute('width');
      const height = await img.getAttribute('height');
      
      // Check for lazy loading
      if (images.indexOf(img) > 2) {
        // Images below the fold should be lazy loaded
        expect(loading).toBe('lazy');
      }
      
      // Check for explicit dimensions (prevents CLS)
      if (src && !src.startsWith('data:')) {
        expect(width || height).toBeTruthy();
      }
    }
  });

  test('no render-blocking resources', async ({ page }) => {
    const resourceTimings: any[] = [];
    
    page.on('response', (response) => {
      const timing = response.timing();
      const url = response.url();
      
      if (url.endsWith('.css') || url.endsWith('.js')) {
        resourceTimings.push({
          url,
          blocked: timing.connectStart - timing.fetchStart,
        });
      }
    });
    
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    // Check for blocking time
    const blockedResources = resourceTimings.filter((r) => r.blocked > 100);
    
    console.log('Blocked resources:', blockedResources.length);
    expect(blockedResources.length).toBe(0);
  });

  test('fonts load efficiently', async ({ page }) => {
    const fontRequests: any[] = [];
    
    page.on('response', (response) => {
      const url = response.url();
      if (url.includes('font') || url.endsWith('.woff2') || url.endsWith('.woff')) {
        fontRequests.push({
          url,
          status: response.status(),
        });
      }
    });
    
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    // Verify fonts loaded successfully
    const failedFonts = fontRequests.filter((f) => f.status !== 200);
    expect(failedFonts).toHaveLength(0);
    
    console.log(`Loaded ${fontRequests.length} fonts`);
  });

  test('no console errors', async ({ page }) => {
    const consoleErrors: string[] = [];
    
    page.on('console', (msg) => {
      if (msg.type() === 'error') {
        consoleErrors.push(msg.text());
      }
    });
    
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    // Should have no console errors
    expect(consoleErrors).toHaveLength(0);
  });
});

