Implement subtask 8002: Configure shadcn/ui component library and design token system

## Objective
Install and configure shadcn/ui with Radix UI primitives. Define the dark/moody design token system in tailwind.config.ts reflecting the Sigma-1 lighting/production company brand: dark palette, accent colors, professional typography, generous spacing.

## Steps
1. Run `npx shadcn@latest init` — select default style, dark theme, CSS variables.
2. Install required shadcn/ui components: Button, Card, Dialog, Form, Input, Select, Table, Tabs, Badge, Calendar, Sheet, Popover, Separator, ScrollArea, Skeleton.
3. Define design tokens in `tailwind.config.ts` / CSS variables:
   - Colors: dark background (#0a0a0a range), muted surfaces (#1a1a1a), accent (warm amber/gold for lighting brand), destructive, muted foreground.
   - Border radius: default 0.5rem.
   - Font family: Inter or similar professional sans-serif via `next/font`.
   - Spacing scale: generous padding/margins for premium feel.
4. Create `components/ui/` barrel exports.
5. Create a `ThemeProvider` if needed (for dark mode toggle future-proofing, but default to dark).
6. Build a style guide page at `/dev/styleguide` (dev only) showing all shadcn components with design tokens applied.
7. Verify all installed components render correctly with the custom theme.

## Validation
Render each installed shadcn/ui component in a test and verify it mounts without errors. Visually confirm design tokens apply (dark background, accent colors) on the styleguide page. Verify `next/font` loads correctly.