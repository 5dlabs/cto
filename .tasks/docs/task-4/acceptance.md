## Acceptance Criteria

- [ ] 1. Environment banner: In a staging build (`NEXT_PUBLIC_ENVIRONMENT=staging`), the `EnvironmentBanner` component renders with text containing 'STAGING' and has an amber-colored background (CSS computed style check). In production build, the banner is not rendered or shows production indicator.
- [ ] 2. Deliberation list: When API returns 3 deliberations, the `/hermes` page renders exactly 3 Card components each containing a deliberation ID and status Badge.
- [ ] 3. Artifact comparison: On `/hermes/[id]`, when a deliberation has 1 current-site screenshot and 2 variants, the comparison view displays the current-site image on the left and a variant selector with 2 thumbnails.
- [ ] 4. Presigned URL loading: Artifact images load successfully from presigned URLs — no CORS errors, images render with correct dimensions.
- [ ] 5. Feature flag: When `NEXT_PUBLIC_HERMES_ENABLED=false`, navigating to `/hermes` does not render the deliberation dashboard and the nav item is absent from the DOM.
- [ ] 6. Accessibility: axe-core audit of `/hermes` and `/hermes/[id]` pages returns zero critical or serious violations. Tab key navigates through all interactive elements in logical order.

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.