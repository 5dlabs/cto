Implement subtask 1010: Write Playwright E2E tests for responsive layout, component presence, and computed styles

## Objective
Implement Playwright E2E tests covering component presence assertions, responsive screenshots at 3 viewports, computed style checks for typographic hierarchy, and vertical ordering validation. These are the core functional tests for the marketing page.

## Steps
1. Create `playwright.config.ts` with:
   - `webServer` directive pointing to `npm run dev` (or `npm run start` after build) on port 3000.
   - Projects for chromium (primary), optionally firefox.
   - Headless mode for CI.
2. Create `e2e/marketing-page.spec.ts` with the following test cases:
3. **Component presence test**:
   - Navigate to `/`.
   - Assert `[data-testid="hero"]`, `[data-testid="features"]`, `[data-testid="cta"]` are all visible.
   - Assert `[data-testid="header"]` and `[data-testid="footer"]` are present.
4. **Vertical ordering test**:
   - Get bounding rects for hero, features, CTA.
   - Assert hero.y < features.y < cta.y.
5. **Responsive screenshot tests**:
   - Set viewport to 375px width, capture full-page screenshot → `screenshots/mobile.png`.
   - Set viewport to 768px width, capture → `screenshots/tablet.png`.
   - Set viewport to 1280px width, capture → `screenshots/desktop.png`.
   - Store as test artifacts.
6. **Computed style checks**:
   - At 1280px viewport: find H1 inside `[data-testid="hero"]`, extract `getComputedStyle(el).fontSize`, parse to number, assert ≥ 36.
   - At 375px viewport: same H1, assert computed font-size ≥ 24.
7. **Features responsive layout test**:
   - At 1280px: verify feature cards have 3 distinct x-positions (3-column layout).
   - At 375px: verify all feature cards have approximately the same x-offset (stacked).
8. Ensure all tests run in headless mode and are CI-compatible.

## Validation
Run `npx playwright test e2e/marketing-page.spec.ts` — all tests pass (exit code 0). Screenshots are generated in expected output directory. Font-size assertions pass at both viewport widths. Vertical ordering assertion passes. Features layout assertion passes at both viewports. Tests complete in <30 seconds.