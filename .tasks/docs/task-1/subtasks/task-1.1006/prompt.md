Implement subtask 1006: Build CTA section component with contrasting background and centered layout

## Objective
Implement `components/CTA.tsx` as a centered call-to-action block with a visually distinct contrasting background color, headline, optional description, and action button. Consume design tokens for all styling.

## Steps
1. Create `components/CTA.tsx` with typed props interface:
   ```typescript
   interface CTAProps {
     headline: string;
     buttonText: string;
     buttonHref: string;
     description?: string;
   }
   ```
2. Render a `<section>` with `data-testid="cta"` and a visually distinct background using design token `accent` or `secondary` color.
3. Center all content horizontally using `text-center max-w-3xl mx-auto` with generous vertical padding (`py-16 md:py-24`).
4. Render an `<h2>` headline with `heading` type scale token, colored with `accentForeground` or appropriate contrasting color.
5. Optional `<p>` description below headline using `body` token.
6. Render a prominent CTA `<a>` styled as a button with contrasting colors. If section background is dark/accent, button should be light. Ensure button text ≥ 4.5:1 contrast against button background, and button background ≥ 3:1 contrast against section background.
7. Button must have visible focus indicator: `focus-visible:ring-2 focus-visible:ring-offset-2`.
8. Responsive: padding and text sizes scale down gracefully via Tailwind responsive utilities.
9. The section background must be visually distinct from the Features section above it to create clear visual separation in the page flow.

## Validation
Playwright test: `[data-testid="cta"]` is visible. Section has a distinct background-color (extract via `getComputedStyle`, verify it's not the same as body/page background). H2 and a link/button element are present within the section. Tab to the button and verify focus ring is visible (check computed `outline` or `box-shadow` properties on `:focus-visible`). Axe-core reports no contrast violations within the CTA section.