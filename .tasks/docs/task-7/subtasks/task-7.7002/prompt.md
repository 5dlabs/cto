Implement subtask 7002: Install and configure shadcn/ui component library

## Objective
Set up shadcn/ui via its CLI, initializing the component system with the default theme. Pre-install the specific shadcn/ui components needed across the dashboard: Table, Button, Input, Alert, Badge, Card, and Skeleton (for loading states).

## Steps
1. Run `npx shadcn-ui@latest init` and select default theme, CSS variables mode, and the `src/components/ui` output directory.
2. Verify `components.json` is created with correct paths and aliases.
3. Install required components: `npx shadcn-ui@latest add table button input alert badge card skeleton`.
4. Verify each component file exists under `src/components/ui/`.
5. Confirm Radix UI peer dependencies are resolved (check `package.json`).
6. Create a smoke-test page at `src/app/test/page.tsx` that renders one of each installed component to verify they work with Tailwind. Remove the test page after verification.

## Validation
All shadcn/ui components render without console errors. `next build` still exits 0 after component installation. Each component file exists under `src/components/ui/`.