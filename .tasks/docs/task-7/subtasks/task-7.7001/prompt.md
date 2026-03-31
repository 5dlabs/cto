Implement subtask 7001: Initialize Playwright project structure and configuration

## Objective
Set up the Playwright test project directory structure, install dependencies, and create playwright.config.ts with cross-browser projects, API request context, retry policy, and reporter configuration.

## Steps
1. Create `tests/e2e/hermes/` directory with `api/` and `browser/` subdirectories.
2. Install `@playwright/test` and `@axe-core/playwright` as dev dependencies.
3. Create `playwright.config.ts` with:
   - `baseURL` from `E2E_BASE_URL` env var
   - 4 projects: `chromium`, `firefox`, `webkit` (browser tests), and `api` (using `request` context only, no browser launch)
   - `retries: process.env.CI ? 2 : 0`
   - `reporter: [['html', { open: 'never' }], ['junit', { outputFile: 'results/junit.xml' }]]`
   - `globalSetup` and `globalTeardown` pointing to setup/teardown files (stubs for now)
   - Trace collection: `on-first-retry`
   - Timeout: 30s per test, 60s navigation timeout
4. Create a `tsconfig.json` for the e2e tests extending the root config.
5. Add npm scripts: `test:e2e:hermes` and `test:e2e:hermes:api`.

## Validation
Run `npx playwright test --list` from the project root and verify it discovers 0 tests without errors, config loads successfully, and all 4 projects are listed.