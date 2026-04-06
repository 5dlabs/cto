Implement subtask 8001: Initialize Next.js 15 project with App Router, React 19, TailwindCSS 4, shadcn/ui, Effect 3.x, and TanStack Query

## Objective
Scaffold the Next.js 15 project with all core dependencies, configure the design system foundation, and set up the API data-fetching layer.

## Steps
1. Run `create-next-app` with Next.js 15, App Router, and TypeScript enabled.
2. Install and configure TailwindCSS 4 with the project's color palette, typography, and spacing tokens for Sigma-1 branding.
3. Install and initialize shadcn/ui; configure the theme (colors, border radius, fonts) to match Sigma-1 brand guidelines.
4. Install Effect 3.x and set up a shared Effect runtime/layer for the application (for form validation, API error handling).
5. Install TanStack Query and configure the QueryClientProvider in the root layout with sensible defaults (staleTime, retry).
6. Create the base layout component (root layout.tsx) with header, footer, navigation, and metadata.
7. Set up the project structure: /app for routes, /components for shared UI, /lib for Effect services and API clients, /hooks for custom hooks.
8. Configure ESLint, Prettier, and TypeScript strict mode.
9. Verify the dev server starts and renders a placeholder home page.

## Validation
Dev server starts without errors; placeholder home page renders; TailwindCSS classes apply correctly; shadcn/ui Button component renders with custom theme; TanStack Query provider is accessible in components; Effect runtime initializes without errors.