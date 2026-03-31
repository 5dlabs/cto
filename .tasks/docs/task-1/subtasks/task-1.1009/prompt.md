Implement subtask 1009: Generate design snapshot task documentation for downstream agents

## Objective
Create `docs/design-snapshot-tasks.md` capturing each component's intent, props API, accessibility requirements, breakpoint expectations, and implementation prompts for downstream agents. This document replaces Stitch candidates as the source of truth for design intent.

## Steps
1. Create `docs/design-snapshot-tasks.md`.
2. Include an **Overview** section explaining:
   - The design snapshot validation flow purpose.
   - That these docs replace Stitch candidates as the design source of truth.
   - Reference to `lib/tokens.ts` as the token authority.
3. For each component (**Hero**, **Features**, **CTA**), create a dedicated `## Component: [Name]` section containing:
   a. **Intent**: What the component achieves visually and functionally (2-3 sentences).
   b. **Props API**: Full TypeScript interface copied from the component, with description of each prop's purpose and default values.
   c. **Accessibility requirements**: Specific contrast ratios required, focus indicator details, semantic HTML elements used, any ARIA attributes.
   d. **Breakpoint behavior**: Describe expected layout and visual output at each breakpoint:
      - `375px (mobile)`: layout, font sizes, padding, stacking behavior.
      - `768px (tablet)`: layout transitions, column changes.
      - `1280px (desktop)`: full layout, max-width constraints, spacing.
   e. **Implementation prompts**: 3-5 actionable bullet points that a downstream agent could use to re-implement or modify the component from scratch.
4. Include a **Design Tokens Reference** section summarizing the token structure (color categories, type scale levels, spacing scale, breakpoints) with a code snippet example.
5. **Critical**: Each component section MUST contain the words 'props', 'accessibility', and 'breakpoint' to pass CI validation.
6. Use clear Markdown formatting: H2 for component names, H3 for subsections, fenced code blocks for TypeScript interfaces, tables for breakpoint comparisons.

## Validation
CI script validates: (1) `docs/design-snapshot-tasks.md` exists. (2) File contains at least 3 sections matching `## Component: Hero`, `## Component: Features`, `## Component: CTA` (or similar H2 patterns). (3) Each component section contains the words 'props', 'accessibility', and 'breakpoint'. (4) File is >5000 characters (substantial content). (5) Markdown linting passes (`npx markdownlint docs/design-snapshot-tasks.md` with no errors, or relaxed rules).