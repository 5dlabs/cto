Implement subtask 1011: Write axe-core accessibility audit and snapshot API validation tests

## Objective
Implement Playwright tests for axe-core accessibility auditing (zero critical/serious violations) and /api/snapshot endpoint validation. Also add a CI validation script for the task documentation file.

## Steps
1. In `e2e/accessibility.spec.ts`:
   - Import `AxeBuilder` from `@axe-core/playwright`.
   - Navigate to `/` at 1280px viewport.
   - Run `new AxeBuilder({ page }).analyze()`.
   - Assert zero `violations` with `impact` of 'critical' or 'serious'.
   - Optionally run at 375px viewport as well to catch mobile-specific issues.
2. In `e2e/snapshot-api.spec.ts`:
   - `GET /api/snapshot` using `page.request.get()`.
   - Assert response status is 200.
   - Parse JSON body.
   - Assert `body.components` deep equals `['Hero', 'Features', 'CTA']`.
   - Assert `body.tokensApplied` is `true`.
3. Create `scripts/validate-docs.sh` (or `scripts/validate-docs.ts` runnable via `npx tsx`):
   - Check `docs/design-snapshot-tasks.md` exists.
   - Check file contains H2 sections for Hero, Features, CTA.
   - For each section, verify presence of words 'props', 'accessibility', 'breakpoint'.
   - Exit with code 0 on success, 1 on failure with descriptive error messages.
4. Add a `validate:docs` script to `package.json` that runs the validation script.
5. Ensure all tests are compatible with the `playwright.config.ts` from subtask 1010.

## Validation
Run `npx playwright test e2e/accessibility.spec.ts` — passes with zero critical/serious axe violations. Run `npx playwright test e2e/snapshot-api.spec.ts` — snapshot API test passes. Run `npm run validate:docs` — exits with code 0 when `docs/design-snapshot-tasks.md` is properly formatted. Final validation: `next build` succeeds and `npx playwright test` (all specs) passes with exit code 0.