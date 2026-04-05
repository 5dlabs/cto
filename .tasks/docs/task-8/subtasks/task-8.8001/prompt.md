Implement subtask 8001: Initialize Next.js 15 project with App Router, React 19, shadcn/ui, TailwindCSS 4, Effect 3.x, and TanStack Query

## Objective
Scaffold the Next.js 15 project with App Router, configure React 19, install and configure shadcn/ui component library, set up TailwindCSS 4, integrate Effect 3.x for typed error handling, and configure TanStack Query for data fetching with Effect integration.

## Steps
Step 1: Run `create-next-app` with Next.js 15 and App Router enabled; configure TypeScript strict mode. Step 2: Install and initialize shadcn/ui — configure theme, color palette (brand colors for Sigma-1), and component defaults. Step 3: Configure TailwindCSS 4 with custom theme tokens (colors, typography, spacing) matching Sigma-1 branding. Step 4: Install Effect 3.x and create a shared Effect runtime/layer for the frontend (API client layer, error types). Step 5: Install and configure TanStack Query with a QueryClientProvider in the root layout; create a custom hook wrapper that integrates TanStack Query with Effect for type-safe API calls. Step 6: Set up the App Router layout structure: root layout with providers, navigation shell, and footer. Step 7: Configure environment variables for API base URLs (from infra ConfigMap). Step 8: Verify the dev server starts and renders a placeholder home page with shadcn/ui components.

## Validation
Dev server starts without errors; TypeScript compilation succeeds with strict mode; shadcn/ui components render with correct theme; TailwindCSS classes apply correctly; TanStack Query provider is mounted; Effect runtime initializes; a test API call via Effect + TanStack Query returns mocked data.