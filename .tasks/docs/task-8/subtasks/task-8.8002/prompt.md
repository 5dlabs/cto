Implement subtask 8002: Set up Sigma-1 design system with TailwindCSS 4 brand tokens and shadcn/ui customization

## Objective
Define the Sigma-1 brand design tokens in tailwind.config.ts (colors, typography, spacing, shadows for a professional dark theme), initialize shadcn/ui with customized CSS variables, and create the foundational UI component set.

## Steps
1. Reference https://deployiq.maximinimal.ca for brand baseline direction.
2. In `tailwind.config.ts`, define custom theme tokens:
   - Colors: primary (professional dark palette — deep navy/charcoal base, accent color for CTAs), secondary, destructive, muted, background, foreground, card, popover.
   - Typography: set `fontFamily.sans` to Inter (import via `next/font/google`).
   - Border radius scale, box shadow tokens.
3. Run `npx shadcn@latest init` and select the custom theme. Update `globals.css` with CSS custom properties matching the brand tokens.
4. Install core shadcn/ui components: Button, Card, Input, Select, Dialog, Sheet, Tabs, Badge, Table, Separator, Skeleton, Toast, Tooltip.
5. Create a `components/sigma1/` directory with placeholder files for custom components: `quote-builder.tsx`, `chat-widget.tsx`, `availability-calendar.tsx`.
6. Build a `components/sigma1/theme-showcase.tsx` page (dev-only) that renders all shadcn/ui components with brand styling for visual QA.
7. Ensure all color tokens pass WCAG 2.1 AA contrast ratio (4.5:1 for normal text, 3:1 for large text) against the dark background.

## Validation
Render the theme showcase page and visually confirm all shadcn/ui components render with Sigma-1 brand tokens. Use a contrast checker tool to verify all text/background combinations meet WCAG AA. Verify Inter font loads correctly via Next.js font optimization.