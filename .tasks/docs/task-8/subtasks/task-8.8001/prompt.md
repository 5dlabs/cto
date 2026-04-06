Implement subtask 8001: Scaffold Next.js 15 project with App Router, React 19, shadcn/ui, TailwindCSS 4, and Effect

## Objective
Initialize the Next.js 15 project with all foundational dependencies, configure App Router, set up TailwindCSS 4, install and configure shadcn/ui components, integrate Effect 3.x, and establish the project's folder structure and shared layout.

## Steps
1. Run `create-next-app` with Next.js 15 and App Router enabled.
2. Install and configure TailwindCSS 4 with the project's design tokens (colors, fonts, spacing).
3. Initialize shadcn/ui: run `npx shadcn-ui@latest init`, configure theme, install base components (Button, Input, Card, Dialog, etc.).
4. Install Effect 3.x (`@effect/schema`, `effect`) and set up shared Effect layers/services for API calls.
5. Create the app directory structure: `app/(marketing)/`, `app/equipment/`, `app/quote/`, `app/portfolio/`.
6. Implement the root layout (`app/layout.tsx`) with global styles, fonts (e.g., Inter), metadata defaults, and a shared Navbar/Footer.
7. Create an API client module that reads backend URLs from environment variables (matching sigma1-infra-endpoints ConfigMap keys).
8. Configure `next.config.js` with image domains, environment variable exposure, and any required rewrites.
9. Add ESLint, Prettier, and TypeScript strict mode configuration.

## Validation
Project builds with `next build` without errors; dev server starts and renders the root layout with navbar and footer; TailwindCSS classes apply correctly; shadcn/ui Button component renders; Effect import works without errors; TypeScript strict mode passes.