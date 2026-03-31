Implement subtask 8002: Set up shadcn/ui component library and shared UI primitives

## Objective
Install and configure shadcn/ui, initialize the component registry, and add the base set of shared UI primitives (Button, Card, Input, Dialog, Sheet, Badge, Skeleton, Separator, NavigationMenu) that will be used across all pages. Resolve dp-12 and dp-13.

## Steps
1. Run `npx shadcn-ui@latest init` and configure `components.json` with the project's style preferences (CSS variables, import alias).
2. Add core shadcn/ui components: Button, Card, Input, Label, Textarea, Select, Dialog, Sheet, Badge, Skeleton, Separator, NavigationMenu, DropdownMenu, Tabs, Toast/Sonner.
3. Resolve dp-12: decide on extension strategy. Create `@/components/ui` for shadcn base and `@/components/custom` for any project-specific extensions.
4. Resolve dp-13: install TanStack Table (`@tanstack/react-table`) if chosen, or set up shadcn/ui Table component. Create a reusable `DataTable` wrapper component with sorting, filtering, and pagination props.
5. Create the main navigation component (Header or Sidebar based on dp-11 resolution from 8001) using shadcn/ui NavigationMenu.
6. Create a Footer component with site links and contact info.
7. Create a shared `PageContainer` layout wrapper for consistent padding/max-width.
8. Set up Sonner or shadcn Toast for global notifications.

## Validation
Render a test page importing at least 5 different shadcn/ui components and verify they display correctly with TailwindCSS 4 styling. Verify the DataTable component renders with mock data, supports sorting, and paginates. Verify navigation component renders all expected links.