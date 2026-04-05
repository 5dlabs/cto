Implement subtask 8001: Initialize Next.js 15 project with App Router, React 19, shadcn/ui, TailwindCSS 4, and Effect 3.x

## Objective
Scaffold the Next.js 15 project with all core dependencies, configure the App Router layout structure, global styles, and shared components (header, footer, navigation).

## Steps
1. Run `create-next-app` with Next.js 15 and App Router enabled.
2. Install and configure TailwindCSS 4 with the project's design tokens (colors, typography, spacing).
3. Install and initialize shadcn/ui: run `npx shadcn-ui@latest init`, configure components.json.
4. Install Effect 3.x (`@effect/io`, `@effect/schema`, `@effect/platform`) and configure TypeScript paths.
5. Create the root layout (`app/layout.tsx`) with html/body, font loading, global CSS.
6. Build shared layout components: Header (logo, nav links to /equipment, /quote, /portfolio), Footer (company info, links), and mobile-responsive navigation.
7. Configure `next.config.js` with image domains (for equipment/portfolio images), environment variables for API endpoints.
8. Set up ESLint and Prettier with the project's conventions.
9. Verify dev server starts and the home page renders the layout shell.

## Validation
Dev server starts without errors. Root layout renders header and footer. TailwindCSS classes apply correctly. shadcn/ui Button component renders. Effect imports resolve without TypeScript errors. Navigation links are present and routable.