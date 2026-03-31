Implement subtask 1007: Compose marketing page by assembling Hero, Features, and CTA in app/page.tsx

## Objective
Assemble Hero → Features → CTA components in `app/page.tsx` to create the canonical marketing validation flow. Provide realistic placeholder content for all component props. Ensure proper section spacing and visual flow.

## Steps
1. In `app/page.tsx`, import Hero, Features, and CTA components.
2. Render them in sequence: Hero at top, Features in middle, CTA at bottom.
3. Provide concrete, realistic marketing copy:
   - Hero: headline (e.g., 'Ship Products That Delight'), subheadline, CTA text + href.
   - Features: 3 features with descriptive titles, body copy, and icon ReactNodes (use the placeholder SVG icons from Features component).
   - CTA: headline (e.g., 'Ready to Get Started?'), button text + href.
4. Ensure consistent vertical spacing between sections using design token spacing values (e.g., `space-y-0` with each section managing its own padding, or explicit gap).
5. The page should create a clear top-to-bottom reading path: hook (Hero) → educate (Features) → convert (CTA).
6. Verify the full page renders without hydration errors or console warnings.
7. Test at all three breakpoints (375px, 768px, 1280px) that the flow is coherent.

## Validation
Playwright E2E: Navigate to `/`, verify Hero, Features, and CTA appear in correct vertical order by comparing `getBoundingClientRect().y` values: Hero.y < Features.y < CTA.y. All three `data-testid` elements are visible. No console errors or React hydration warnings. `next build` succeeds with this page composition. Page renders correctly at 375px, 768px, and 1280px viewports (visual spot check via screenshots).