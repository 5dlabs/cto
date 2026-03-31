Implement subtask 4001: Implement EnvironmentBanner component with CSS custom property theming

## Objective
Create `components/EnvironmentBanner.tsx` that reads `NEXT_PUBLIC_ENVIRONMENT` and renders a staging/production-aware banner, plus wire up CSS custom property theming for `--accent-color` across environments.

## Steps
1. Create `components/EnvironmentBanner.tsx`:
   - Read `process.env.NEXT_PUBLIC_ENVIRONMENT` at render time
   - When value is not `'production'`: render a fixed/sticky top bar with `⚠ STAGING` text, amber background (`bg-amber-500`), white text, and an accessible text label (not color-only per WCAG)
   - When `'production'`: render nothing (return null) or a subtle, minimal production indicator
   - Component should be a client component (`'use client'`) if using any client-side logic, otherwise server component is fine
2. CSS custom property theming in `app/globals.css` or `tailwind.config.ts`:
   - Define `:root { --accent-color: <brand-color>; }` for production
   - Define `[data-environment='staging'] { --accent-color: theme(colors.amber.500); }` or use a body class approach
   - Set `data-environment` attribute on `<html>` or `<body>` in root layout based on `NEXT_PUBLIC_ENVIRONMENT`
3. Add `<EnvironmentBanner />` to `app/layout.tsx` as the first child in the body, before any other content.
4. Ensure the banner accounts for layout shift — use a fixed height so content below doesn't jump.

## Validation
Unit test: render `EnvironmentBanner` with `NEXT_PUBLIC_ENVIRONMENT=staging`, assert text 'STAGING' is present and the container has amber background class. Render with `NEXT_PUBLIC_ENVIRONMENT=production`, assert the banner is not in the DOM (or shows production indicator). Verify `--accent-color` CSS variable resolves to the correct value in each environment. Accessibility: verify the banner has a text label, not just color.