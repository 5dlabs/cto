Implement subtask 6001: Initialize Next.js application with shadcn/ui setup

## Objective
Set up or extend the Next.js project with shadcn/ui component library, Tailwind CSS configuration, and project structure for the pipeline dashboard feature.

## Steps
1. If no Next.js app exists, run `npx create-next-app@latest` with App Router enabled and TypeScript.
2. Install and initialize shadcn/ui: `npx shadcn-ui@latest init` — configure tailwind.config.ts, globals.css, and component paths.
3. If the team's tweakcn configuration is accessible, apply it for consistent theming. Otherwise, use shadcn/ui defaults with the neutral color palette.
4. Install required shadcn/ui components: `npx shadcn-ui@latest add card badge avatar`.
5. Set up the project directory structure: `app/pipeline/[sessionId]/page.tsx`, `components/pipeline/`, `lib/api/`.
6. Configure environment variable reading for `NEXT_PUBLIC_PM_SERVER_URL` (sourced from `sigma-1-infra-endpoints` ConfigMap via `envFrom` at the pod level).
7. Verify the dev server starts and renders a placeholder page at `/pipeline/test`.

## Validation
Verify: (1) `npm run dev` starts without errors, (2) navigating to `/pipeline/test` renders the placeholder page, (3) shadcn/ui Card component renders correctly when imported into a test page.