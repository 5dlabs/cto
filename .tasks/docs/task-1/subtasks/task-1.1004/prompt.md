Implement subtask 1004: Build Hero section component with headline, sub-headline, and CTA button

## Objective
Implement `components/Hero.tsx` as a full-width hero section with H1 headline, H2 sub-headline, primary CTA button, and optional background gradient. Consume design tokens for all typography and color values. Enforce font-size ratio ≥ 1.5× between H1 and body text.

## Steps
1. Create `components/Hero.tsx` as a React component with typed props interface:
   ```typescript
   interface HeroProps {
     headline: string;
     subheadline: string;
     ctaText: string;
     ctaHref: string;
     backgroundGradient?: string;
   }
   ```
2. Render a full-width `<section>` with `data-testid="hero"`.
3. Use an `<h1>` for the headline styled with the `display` type scale token (48px desktop / 28px mobile via Tailwind responsive classes).
4. Use an `<h2>` for the sub-headline styled with `subheading` token (24px desktop / 18px mobile).
5. Render a primary CTA as an `<a>` element styled as a button using design token `primary` / `primaryForeground` colors with sufficient padding (`px-6 py-3`), rounded corners, and hover state.
6. The CTA must have a visible focus indicator: `focus-visible:ring-2 focus-visible:ring-offset-2` or equivalent.
7. Apply a background gradient using design token colors if `backgroundGradient` prop is not provided (default to a subtle gradient from `background` to `surface`).
8. Responsive: center text on all breakpoints, constrain content width to `max-w-4xl mx-auto`, generous vertical padding (`py-20 md:py-32`).
9. Verify computed font-size ratio: display (48px) / body (16px) = 3× at desktop, display (28px) / body (16px) = 1.75× at mobile — both ≥ 1.5× ✓.
10. All colors must meet 4.5:1 contrast against their backgrounds.

## Validation
Playwright test: `[data-testid="hero"]` is visible. H1 element exists within hero with non-empty text. At 1280px viewport, H1 computed font-size ≥ 36px (token says 48px). At 375px viewport, H1 computed font-size ≥ 24px (token says 28px). CTA link is visible and has `href` attribute. Axe-core reports no violations within the hero section. Focus the CTA element and verify visible outline/ring via computed styles.