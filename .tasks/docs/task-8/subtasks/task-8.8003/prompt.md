<identity>
You are blaze working on subtask 8003 of task 8.
</identity>

<context>
<scope>
Run shadcn/ui initialization configured for New York style and TailwindCSS 4, then install all component primitives needed across the site.
</scope>
</context>

<implementation_plan>
1. `cd apps/website && npx shadcn-ui@latest init` — select: New York style, TailwindCSS 4, CSS variables yes, TypeScript yes.
2. Install components one by one (shadcn installs into components/ui/):
   `npx shadcn-ui@latest add button card badge sheet sidebar dialog select input skeleton avatar table tabs`
3. Verify each component file exists in apps/website/components/ui/.
4. Ensure globals.css has shadcn CSS variable definitions and they map correctly to the design token colors (update --background, --foreground, --primary, --accent to match sigma1 palette from design-tokens package).
5. Test render: create a throwaway page that uses Button, Card, Badge to confirm no runtime errors.
</implementation_plan>

<validation>
All 12 component files exist in components/ui/. `bun run build` exits 0. A test page importing Button, Card, Badge, Sheet, Sidebar, Dialog, Select, Input, Skeleton, Avatar, Table, Tabs renders without TypeScript errors.
</validation>