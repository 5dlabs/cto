Implement subtask 8001: Scaffold Next.js 15 project with App Router, React 19, TailwindCSS 4, shadcn/ui, Effect, and TanStack Query

## Objective
Initialize the Next.js 15 project with all foundational dependencies configured: App Router, React 19, TailwindCSS 4, shadcn/ui component library, Effect 3.x, and TanStack Query for data fetching.

## Steps
1. Run `create-next-app` with Next.js 15 and App Router enabled.
2. Install and configure React 19.
3. Install and configure TailwindCSS 4 with the project's design tokens (colors, typography, spacing for Sigma-1 brand).
4. Initialize shadcn/ui and install base components (Button, Card, Input, Dialog, Sheet, etc.).
5. Install Effect 3.x and set up the Effect runtime/layer structure for the app (e.g., src/lib/effect/).
6. Install and configure TanStack Query provider wrapping the app layout.
7. Set up the App Router layout structure: root layout with metadata, global providers (QueryClientProvider, ThemeProvider).
8. Configure TypeScript strict mode, path aliases (@/), and ESLint/Prettier.
9. Set up environment variables for API endpoints (referencing sigma1-infra-endpoints).
10. Create a basic health/smoke page to verify the stack works end-to-end.

## Validation
Project builds without errors; dev server starts and renders the root layout; TailwindCSS classes apply correctly; shadcn/ui Button component renders; Effect runtime initializes; TanStack Query devtools appear in dev mode; TypeScript compilation passes with strict mode.