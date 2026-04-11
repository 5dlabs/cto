# Playwright Testing

End-to-end browser testing with Playwright for web applications.

## When to Use

- Setting up browser automation tests
- Testing user flows and interactions
- Cross-browser testing (Chromium, Firefox, WebKit)
- Visual regression testing
- API testing alongside UI tests

## Quick Start

### Installation

```bash
npm init playwright@latest
# Or
pnpm create playwright
```

### Basic Test Structure

```typescript
import { test, expect } from '@playwright/test';

test('homepage has title', async ({ page }) => {
  await page.goto('https://example.com');
  await expect(page).toHaveTitle(/Example/);
});

test('user can login', async ({ page }) => {
  await page.goto('/login');
  await page.fill('[name="email"]', 'user@example.com');
  await page.fill('[name="password"]', 'password123');
  await page.click('button[type="submit"]');
  await expect(page).toHaveURL('/dashboard');
});
```

## Configuration

### playwright.config.ts

```typescript
import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
  testDir: './tests',
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 1 : undefined,
  reporter: 'html',
  use: {
    baseURL: 'http://localhost:3000',
    trace: 'on-first-retry',
    screenshot: 'only-on-failure',
  },
  projects: [
    { name: 'chromium', use: { ...devices['Desktop Chrome'] } },
    { name: 'firefox', use: { ...devices['Desktop Firefox'] } },
    { name: 'webkit', use: { ...devices['Desktop Safari'] } },
  ],
  webServer: {
    command: 'npm run dev',
    url: 'http://localhost:3000',
    reuseExistingServer: !process.env.CI,
  },
});
```

## Common Patterns

### Page Object Model

```typescript
// pages/login.page.ts
import { Page, Locator } from '@playwright/test';

export class LoginPage {
  readonly page: Page;
  readonly emailInput: Locator;
  readonly passwordInput: Locator;
  readonly submitButton: Locator;

  constructor(page: Page) {
    this.page = page;
    this.emailInput = page.locator('[name="email"]');
    this.passwordInput = page.locator('[name="password"]');
    this.submitButton = page.locator('button[type="submit"]');
  }

  async goto() {
    await this.page.goto('/login');
  }

  async login(email: string, password: string) {
    await this.emailInput.fill(email);
    await this.passwordInput.fill(password);
    await this.submitButton.click();
  }
}
```

### Using Page Objects

```typescript
import { test, expect } from '@playwright/test';
import { LoginPage } from './pages/login.page';

test('user can login', async ({ page }) => {
  const loginPage = new LoginPage(page);
  await loginPage.goto();
  await loginPage.login('user@example.com', 'password');
  await expect(page).toHaveURL('/dashboard');
});
```

### Fixtures for Authentication

```typescript
// fixtures.ts
import { test as base } from '@playwright/test';

export const test = base.extend({
  authenticatedPage: async ({ page }, use) => {
    await page.goto('/login');
    await page.fill('[name="email"]', 'test@example.com');
    await page.fill('[name="password"]', 'password');
    await page.click('button[type="submit"]');
    await page.waitForURL('/dashboard');
    await use(page);
  },
});

// Usage
test('authenticated user sees dashboard', async ({ authenticatedPage }) => {
  await expect(authenticatedPage.locator('h1')).toHaveText('Dashboard');
});
```

### API Testing

```typescript
import { test, expect } from '@playwright/test';

test('API returns users', async ({ request }) => {
  const response = await request.get('/api/users');
  expect(response.ok()).toBeTruthy();
  
  const users = await response.json();
  expect(users).toHaveLength(10);
});

test('can create user via API', async ({ request }) => {
  const response = await request.post('/api/users', {
    data: { name: 'Test User', email: 'test@example.com' }
  });
  expect(response.status()).toBe(201);
});
```

## Selectors

### Preferred Selectors (in order)

1. **Role-based**: `page.getByRole('button', { name: 'Submit' })`
2. **Text-based**: `page.getByText('Welcome')`
3. **Label-based**: `page.getByLabel('Email')`
4. **Placeholder**: `page.getByPlaceholder('Enter email')`
5. **Test ID**: `page.getByTestId('submit-button')`
6. **CSS**: `page.locator('.submit-btn')` (last resort)

### Examples

```typescript
// Prefer these
await page.getByRole('button', { name: 'Sign In' }).click();
await page.getByLabel('Email address').fill('test@example.com');
await page.getByPlaceholder('Search...').fill('query');
await page.getByTestId('user-menu').click();

// Avoid unless necessary
await page.locator('#submit-btn').click();
await page.locator('button.primary').click();
```

## Running Tests

```bash
# Run all tests
npx playwright test

# Run specific file
npx playwright test login.spec.ts

# Run with UI
npx playwright test --ui

# Debug mode
npx playwright test --debug

# Generate report
npx playwright show-report
```

## Best Practices

1. **Use web-first assertions**: `await expect(locator).toBeVisible()` waits automatically
2. **Avoid hard waits**: Use `waitForURL`, `waitForResponse` instead of `page.waitForTimeout`
3. **Use page objects**: Keep tests clean and DRY
4. **Test isolation**: Each test should be independent
5. **Meaningful test names**: Describe the user behavior being tested
6. **Trace on failure**: Configure `trace: 'on-first-retry'` for debugging
