Implement subtask 7006: Implement browser test suite for artifact comparison, artifact viewer, and accessibility

## Objective
Create Playwright browser specs for the artifact comparison view, artifact viewer dialog interaction, and axe-core accessibility audits on Hermes pages.

## Steps
1. Create `tests/e2e/hermes/browser/artifact-comparison.spec.ts`:
   - Load `fullAccessUser` storageState
   - Navigate to a completed deliberation detail page (use ID from `.testdata/ids.json`)
   - Assert comparison view is visible with a current-site screenshot image
   - Assert image element has `naturalWidth > 0` and `naturalHeight > 0` (via `evaluate`)
   - Click a variant thumbnail → assert the variant image loads in the comparison view
   - Assert both current-site and variant images have non-zero dimensions

2. Create `tests/e2e/hermes/browser/artifact-viewer.spec.ts`:
   - Navigate to completed deliberation detail page
   - Click an artifact thumbnail/image → assert a dialog/modal opens
   - Assert the dialog contains a full-size image with `naturalWidth > 0`
   - Assert a download button is present and has a valid `href` or `onclick` handler
   - Click download button → assert no network error (intercept download request, verify 200)
   - Press Escape key → assert dialog is no longer visible

3. Create `tests/e2e/hermes/browser/accessibility.spec.ts`:
   - Import `AxeBuilder` from `@axe-core/playwright`
   - Test: Navigate to `/hermes`, run `new AxeBuilder({ page }).analyze()` → assert `violations.filter(v => v.impact === 'critical' || v.impact === 'serious').length === 0`
   - Test: Navigate to `/hermes/{completedDeliberationId}`, run axe → same assertion
   - Log all violations (including minor) for informational purposes

4. Use `page.waitForLoadState('networkidle')` before running axe to ensure all content is rendered.

## Validation
Run specs across chromium, firefox, webkit. Comparison view loads two images with non-zero dimensions. Viewer dialog opens/closes correctly with Escape. Download button triggers a successful request. Axe-core reports zero critical/serious violations on both `/hermes` and `/hermes/[id]`. All pass in all 3 browsers.