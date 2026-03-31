## Acceptance Criteria

- [ ] 1. **Visual hierarchy assertion**: Run axe-core accessibility audit via `next build && npx axe-cli http://localhost:3000` — zero critical or serious violations.
- [ ] 2. **Responsive layout**: Use Playwright to capture screenshots at 375px, 768px, and 1280px viewports; verify Hero H1 font-size ≥ 36px at desktop and ≥ 24px at mobile by inspecting computed styles.
- [ ] 3. **Component render**: Playwright E2E test navigates to `/`, asserts presence of `[data-testid="hero"]`, `[data-testid="features"]`, `[data-testid="cta"]` — all three must be visible in viewport sequence.
- [ ] 4. **Snapshot API**: `GET /api/snapshot` returns 200 with JSON containing `{ components: ["Hero", "Features", "CTA"], tokensApplied: true }`.
- [ ] 5. **Task doc completeness**: CI script validates `docs/design-snapshot-tasks.md` exists, contains at least 3 component sections (Hero, Features, CTA), and each section includes the words 'props', 'accessibility', and 'breakpoint'.
- [ ] 6. **Build success**: `next build` completes with zero errors and zero TypeScript type errors.

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.