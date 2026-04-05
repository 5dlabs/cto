Implement subtask 8015: Write E2E test suite with Playwright for critical user flows and Lighthouse performance validation

## Objective
Create comprehensive Playwright E2E tests covering the primary user journey (Home → Equipment → Product → Quote → Submit) and Lighthouse performance audits ensuring the home page scores >= 90 on Performance, Accessibility, and SEO.

## Steps
1. Configure Playwright in `playwright.config.ts`: base URL, browsers (chromium, firefox, webkit), screenshot on failure.
2. Create E2E test: `tests/e2e/quote-flow.spec.ts`:
   - Navigate to /.
   - Click 'Browse Equipment' CTA → verify redirect to /equipment.
   - Verify equipment table renders with items.
   - Click first product → verify redirect to /equipment/[id].
   - Verify product detail page renders with name, price, specs.
   - Click 'Add to Quote' → verify toast notification.
   - Navigate to /quote → verify item is pre-populated in Step 2.
   - Fill Step 1 (event details), advance.
   - Verify pre-populated item in Step 2, add one more item, advance.
   - Step 3: verify review shows all items with pricing.
   - Step 4: fill contact info, submit.
   - Verify success confirmation is displayed.
   - Intercept the POST request and assert payload structure.
3. Create E2E test: `tests/e2e/chat-widget.spec.ts`:
   - Open chat widget on home page.
   - Send a message, verify it appears as user bubble.
   - Navigate to /equipment (via nav link), verify chat widget is still open with message history.
4. Create Lighthouse test: `tests/lighthouse.spec.ts`:
   - Use `@playwright-community/lighthouse` or run Lighthouse CI.
   - Assert / page scores: Performance >= 90, Accessibility >= 90, SEO >= 90.
5. Create test: `tests/e2e/llms-txt.spec.ts`:
   - GET /llms.txt → assert status 200, content-type text/plain, body contains expected keywords.

## Validation
Run `npx playwright test` and verify all E2E tests pass. The quote flow test must complete the full Home → Equipment → Product → Quote → Submit journey. Chat persistence test must verify widget survives navigation. Lighthouse test must achieve >= 90 on all three scores. llms.txt test must validate response format and content.