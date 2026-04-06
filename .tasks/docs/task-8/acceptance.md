## Acceptance Criteria

- [ ] 1. next build completes without TypeScript errors or missing module errors. 2. GET / renders hero section with CTA button linking to /equipment and /quote (Playwright snapshot test). 3. GET /equipment displays product grid; filter by category 'Lighting' shows only lighting products (Playwright interaction test). 4. GET /equipment/:id with a valid product ID shows product name, day_rate, and availability date picker; selecting dates within 7 days updates availability badge without full page reload (Playwright assertion on DOM change). 5. POST quote form on /quote with valid data creates opportunity — mock API returns 201, UI shows confirmation panel with quote ID. 6. GET /llms.txt returns Content-Type: text/plain and body contains company name Sigma-1. 7. Lighthouse score on / >= 90 for Performance and Accessibility (measured via CI Lighthouse action). 8. Morgan chat widget: chat button visible on all pages, clicking opens panel, panel renders iframe/widget pointing to NEXT_PUBLIC_MORGAN_WIDGET_URL.

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.