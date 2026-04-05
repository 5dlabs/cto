Implement subtask 8014: Implement WCAG 2.1 AA accessibility compliance and axe-core testing setup

## Objective
Audit and fix all pages and components for WCAG 2.1 AA compliance, set up axe-core automated accessibility testing, and configure Playwright tests for accessibility validation on key pages.

## Steps
1. Install `@axe-core/playwright` for Playwright-based accessibility testing.
2. Create Playwright accessibility test file `tests/accessibility.spec.ts`:
   - Test /, /equipment, /quote, /portfolio pages.
   - For each: navigate to page, run `axe.analyze()`, assert zero critical and zero serious violations.
3. Audit and fix common issues across all pages:
   - Ensure all images have alt text (including equipment images, portfolio images).
   - Ensure all form inputs have associated labels (quote builder, search inputs, chat input).
   - Ensure color contrast meets 4.5:1 for normal text, 3:1 for large text (already addressed in design system, but verify in context).
   - Ensure focus indicators are visible on all interactive elements.
   - Ensure heading hierarchy is correct (h1 → h2 → h3, no skips).
   - Ensure skip-to-main-content link is present.
4. Chat widget accessibility (verify from 8009):
   - role='dialog', aria-label, aria-live='polite' on message area.
   - Keyboard: Escape to close, Tab to navigate.
5. Equipment table accessibility:
   - Proper table headers with scope attributes.
   - Pagination controls labeled.
6. Quote builder accessibility:
   - Form step announcements via aria-live region.
   - Error messages associated with fields via aria-describedby.

## Validation
Run Playwright accessibility tests on /, /equipment, /quote, /portfolio — all must report zero critical and zero serious axe-core violations. Manually verify keyboard navigation through the entire quote builder flow (Tab through all steps). Verify screen reader announces chat widget messages via aria-live region (test with VoiceOver or NVDA simulator).