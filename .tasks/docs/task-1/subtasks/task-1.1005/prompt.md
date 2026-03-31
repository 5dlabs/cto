Implement subtask 1005: Build Features grid component with responsive 3-column layout

## Objective
Implement `components/Features.tsx` as a responsive 3-column grid that stacks to 1 column on mobile. Each card includes an icon slot (ReactNode), H3 title, and description paragraph. Consume design tokens for typography, spacing, and colors.

## Steps
1. Create `components/Features.tsx` with typed props interface:
   ```typescript
   interface Feature {
     icon: React.ReactNode;
     title: string;
     description: string;
   }
   interface FeaturesProps {
     sectionTitle?: string;
     features: Feature[];
   }
   ```
2. Render a `<section>` with `data-testid="features"`.
3. Add an optional section heading `<h2>` (default: 'Features') using the `heading` type scale token.
4. Use CSS Grid: `grid grid-cols-1 md:grid-cols-3 gap-8` for the responsive layout.
5. Each card renders: the icon `ReactNode`, an `<h3>` title using `subheading` token, and a `<p>` description using `body` token.
6. Cards should have consistent padding from spacing tokens (`p-6`), optional subtle border (`border border-gray-200`) or shadow.
7. All text must meet 4.5:1 contrast ratio against card background.
8. Responsive behavior: 375px → single column. 768px → 2 or 3 columns. 1280px → 3 columns.
9. Provide a default set of 3 placeholder features with simple SVG icons for development/demo purposes (e.g., lightning bolt, shield, chart icons as inline SVGs).
10. Center the section content with `max-w-6xl mx-auto` and generous vertical padding.

## Validation
Playwright test: `[data-testid="features"]` is visible. At 1280px viewport, 3 feature cards are rendered — verify by counting elements matching a card selector and comparing bounding box x-positions (3 distinct x values). At 375px, verify cards stack vertically (all cards have approximately the same x-offset). Each card contains an H3 and a paragraph element. Axe-core reports no violations within the features section.