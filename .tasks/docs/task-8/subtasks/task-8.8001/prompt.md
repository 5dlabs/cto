Implement subtask 8001: Initialize Next.js 15 project with TypeScript, TailwindCSS 4, and core dependencies

## Objective
Scaffold the Next.js 15 App Router project, install all required dependencies (shadcn/ui, Effect 3.x, TanStack Query, TanStack Table v8, react-hook-form v7, @hookform/resolvers), and configure biome.js alongside ESLint for linting.

## Steps
1. Run `npx create-next-app@15` with TypeScript, TailwindCSS 4, ESLint, and App Router options enabled.
2. Install production dependencies: `@tanstack/react-query`, `@tanstack/react-table`, `react-hook-form`, `@hookform/resolvers`, `effect` (3.x), `zod` (for hookform resolver).
3. Install dev dependencies: `@biomejs/biome`, `playwright`, `@axe-core/react`.
4. Configure `biome.json` with formatting and linting rules that complement the existing ESLint config (disable overlapping rules).
5. Set up `tsconfig.json` path aliases: `@/components`, `@/lib`, `@/hooks`, `@/app`.
6. Create the folder structure: `app/`, `components/ui/`, `components/sigma1/`, `lib/`, `hooks/`, `public/`.
7. Add `.env.local.example` with placeholder API base URL variables (NEXT_PUBLIC_API_BASE_URL, NEXT_PUBLIC_WS_URL).
8. Verify `next dev` runs without errors and the default page renders.

## Validation
Run `next dev` and confirm the app starts without errors. Run `biome check .` and confirm no lint errors on the scaffolded code. Verify all dependencies are resolvable via `tsc --noEmit`.