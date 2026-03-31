## Implement Marketing Website Design Snapshot Validation Flow (Blaze - React/Next.js)

### Objective
Build the marketing website frontend that validates the design snapshot E2E flow. This includes establishing the Next.js project structure, implementing a clear visual hierarchy across key marketing pages (hero, features, CTA sections), and producing exportable task documentation/prompts suitable for downstream implementation agents. Since Stitch candidate generation failed, the implementation must be driven directly from design-intent principles: strong typographic hierarchy, responsive layout, accessible contrast ratios, and semantic HTML structure.

### Ownership
- Agent: blaze
- Stack: React/Next.js
- Priority: high
- Status: pending
- Dependencies: None

### Implementation Details
1. **Project scaffold**: Initialize a Next.js 14 (App Router) project with TypeScript, Tailwind CSS 3.4+, and ESLint/Prettier.
2. **Design tokens**: Create a `tokens.ts` file defining color palette, type scale (minimum 4 levels: display, heading, subheading, body), spacing scale, and breakpoints. These act as the single source of truth since no Stitch candidates were produced.
3. **Layout shell**: Implement `app/layout.tsx` with a sticky header (logo + nav), main content slot, and footer. Use semantic HTML (`<header>`, `<main>`, `<footer>`).
4. **Hero section component** (`components/Hero.tsx`): Full-width hero with headline (H1), sub-headline (H2), primary CTA button, and optional background image/gradient. Ensure hierarchy via font-size ratio ≥ 1.5× between H1 and body.
5. **Features grid component** (`components/Features.tsx`): 3-column responsive grid (stacks to 1-col on mobile) with icon, title (H3), and description per card.
6. **CTA section component** (`components/CTA.tsx`): Centered call-to-action block with contrasting background, headline, and button.
7. **Design snapshot validation page** (`app/page.tsx`): Compose Hero → Features → CTA to represent the canonical marketing flow.
8. **Task doc generation**: Create `docs/design-snapshot-tasks.md` capturing each component's intent, props API, accessibility requirements, and implementation prompts that downstream agents can consume. Include screenshots or descriptions of expected visual output at each breakpoint (mobile 375px, tablet 768px, desktop 1280px).
9. **Accessibility**: All interactive elements must have visible focus indicators; color contrast ≥ 4.5:1 (AA); landmark roles present.
10. **Export validation**: Add a `/api/snapshot` route handler that returns a JSON summary of the page's component tree and design token usage for E2E verification.

### Subtasks
- [ ] Initialize Next.js 14 App Router project with TypeScript, Tailwind CSS, and linting: Scaffold the Next.js 14 project using App Router, configure TypeScript strict mode, install and configure Tailwind CSS 3.4+, and set up ESLint and Prettier with consistent rules. This is the foundation that all other subtasks depend on.
- [ ] Create design tokens file with color palette, type scale, spacing, and breakpoints: Define the `lib/tokens.ts` file that serves as the single source of truth for all design decisions, replacing absent Stitch candidates. Export typed constants for color palette (with verified AA contrast pairings), 4-level type scale, spacing scale, and responsive breakpoints. Extend Tailwind config to consume these tokens.
- [ ] Implement layout shell with sticky header, main content slot, and footer: Build `app/layout.tsx` with a sticky header containing logo and navigation, a `<main>` content slot, and a semantic `<footer>`. Use semantic HTML landmarks throughout. Configure font loading via next/font.
- [ ] Build Hero section component with headline, sub-headline, and CTA button: Implement `components/Hero.tsx` as a full-width hero section with H1 headline, H2 sub-headline, primary CTA button, and optional background gradient. Consume design tokens for all typography and color values. Enforce font-size ratio ≥ 1.5× between H1 and body text.
- [ ] Build Features grid component with responsive 3-column layout: Implement `components/Features.tsx` as a responsive 3-column grid that stacks to 1 column on mobile. Each card includes an icon slot (ReactNode), H3 title, and description paragraph. Consume design tokens for typography, spacing, and colors.
- [ ] Build CTA section component with contrasting background and centered layout: Implement `components/CTA.tsx` as a centered call-to-action block with a visually distinct contrasting background color, headline, optional description, and action button. Consume design tokens for all styling.
- [ ] Compose marketing page by assembling Hero, Features, and CTA in app/page.tsx: Assemble Hero → Features → CTA components in `app/page.tsx` to create the canonical marketing validation flow. Provide realistic placeholder content for all component props. Ensure proper section spacing and visual flow.
- [ ] Implement /api/snapshot route handler for E2E validation: Create the `/api/snapshot` Next.js Route Handler that returns a JSON summary of the page's component tree and design token usage. This route serves as the machine-readable validation endpoint for the design snapshot flow.
- [ ] Generate design snapshot task documentation for downstream agents: Create `docs/design-snapshot-tasks.md` capturing each component's intent, props API, accessibility requirements, breakpoint expectations, and implementation prompts for downstream agents. This document replaces Stitch candidates as the source of truth for design intent.
- [ ] Write Playwright E2E tests for responsive layout, component presence, and computed styles: Implement Playwright E2E tests covering component presence assertions, responsive screenshots at 3 viewports, computed style checks for typographic hierarchy, and vertical ordering validation. These are the core functional tests for the marketing page.
- [ ] Write axe-core accessibility audit and snapshot API validation tests: Implement Playwright tests for axe-core accessibility auditing (zero critical/serious violations) and /api/snapshot endpoint validation. Also add a CI validation script for the task documentation file.