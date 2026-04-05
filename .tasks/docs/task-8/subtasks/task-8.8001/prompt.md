Implement subtask 8001: Initialize Next.js 15 project with App Router, React 19, shadcn/ui, TailwindCSS 4, Effect 3.x, and Cloudflare Pages deployment

## Objective
Bootstrap the Next.js 15 project with all required dependencies, configure the App Router, set up the design system foundation (shadcn/ui + TailwindCSS 4), integrate Effect 3.x, configure TanStack Query, and establish the Cloudflare Pages deployment pipeline.

## Steps
1. Create a new Next.js 15 project with the App Router enabled and React 19.
2. Install and configure TailwindCSS 4 with the project's custom theme tokens/colors.
3. Install and initialize shadcn/ui: run the CLI init, configure components.json, and add base components (Button, Card, Input, Dialog, Sheet).
4. Install Effect 3.x and set up Effect.Schema for shared validation schemas (e.g., equipment item, quote request, contact form).
5. Install and configure TanStack Query (React Query) with a global QueryClientProvider in the root layout.
6. Set up the App Router layout structure: root layout with metadata, navigation shell, and footer.
7. Configure the Cloudflare Pages adapter (`@cloudflare/next-on-pages` or equivalent) and create a `wrangler.toml` for deployment.
8. Set up CI/CD: configure a deployment script or GitHub Actions workflow that builds and deploys to Cloudflare Pages on push to main.
9. Verify the project builds, runs locally, and deploys to a preview URL on Cloudflare Pages.

## Validation
Project builds without errors locally; `next dev` serves the app on localhost; shadcn/ui components render correctly; TailwindCSS 4 utility classes apply; Effect.Schema can validate a sample object; TanStack Query provider is accessible in components; Cloudflare Pages deployment succeeds and the preview URL loads the app.