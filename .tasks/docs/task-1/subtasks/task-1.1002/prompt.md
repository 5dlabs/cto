Implement subtask 1002: Create design tokens file with color palette, type scale, spacing, and breakpoints

## Objective
Define the `lib/tokens.ts` file that serves as the single source of truth for all design decisions, replacing absent Stitch candidates. Export typed constants for color palette (with verified AA contrast pairings), 4-level type scale, spacing scale, and responsive breakpoints. Extend Tailwind config to consume these tokens.

## Steps
1. Create `lib/tokens.ts`.
2. Define a `colors` object with at least: `primary`, `primaryForeground`, `secondary`, `secondaryForeground`, `accent`, `accentForeground`, `background`, `surface`, `text`, `textMuted`. All foreground/background pairings must meet 4.5:1 contrast ratio (AA). Document each pairing's contrast ratio in a JSDoc comment.
3. Define a `typeScale` object with 4 levels:
   - `display`: { desktop: '48px', mobile: '28px', lineHeight: '1.1', fontWeight: '700' }
   - `heading`: { desktop: '36px', mobile: '24px', lineHeight: '1.2', fontWeight: '600' }
   - `subheading`: { desktop: '24px', mobile: '18px', lineHeight: '1.3', fontWeight: '500' }
   - `body`: { desktop: '16px', mobile: '16px', lineHeight: '1.6', fontWeight: '400' }
4. Ensure display-to-body ratio is ≥ 1.5× at both desktop and mobile (48/16=3× ✓, 28/16=1.75× ✓).
5. Define a `spacing` scale: `{ xs: '4px', sm: '8px', md: '16px', lg: '24px', xl: '32px', '2xl': '48px', '3xl': '64px' }`.
6. Define `breakpoints`: `{ mobile: '375px', tablet: '768px', desktop: '1280px' }`.
7. Extend `tailwind.config.ts` to consume these tokens: add custom colors, fontSize entries, and spacing to `theme.extend`. Based on the decision point resolution for token strategy (default: Tailwind config extension at build time).
8. Export all tokens with full TypeScript typing and JSDoc comments.

## Validation
`lib/tokens.ts` exports `colors`, `typeScale`, `spacing`, and `breakpoints` objects. `npx tsc --noEmit` passes. Tailwind config references tokens — verify by grep: `tailwind.config.ts` contains keys from `tokens.colors`. Manual or scripted contrast check: primary text on background ≥ 4.5:1 ratio (can use `wcag-contrast` npm package in a test script). Display font-size / body font-size ≥ 1.5 at both desktop and mobile sizes.