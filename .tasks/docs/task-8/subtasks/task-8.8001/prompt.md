Implement subtask 8001: Scaffold Next.js 15 project with App Router, React 19, TailwindCSS 4, and Effect 3.x

## Objective
Initialize the Next.js 15 project using App Router, configure React 19, install and configure TailwindCSS 4, and set up Effect 3.x with the project's base TypeScript configuration. Establish the root layout, global styles, font loading, and base metadata.

## Steps
1. Run `npx create-next-app@latest` with App Router enabled and TypeScript.
2. Install TailwindCSS 4 and configure `tailwind.config.ts` with the project's design tokens (colors, spacing, fonts).
3. Install Effect 3.x (`effect`, `@effect/schema`, `@effect/platform`).
4. Create `app/layout.tsx` with global HTML structure, metadata defaults (title, description, viewport), and font imports.
5. Create `app/globals.css` with TailwindCSS directives and any CSS custom properties.
6. Set up `tsconfig.json` path aliases (`@/components`, `@/lib`, `@/services`).
7. Configure `next.config.ts` with strict mode, image optimization domains, and any environment variable exposure.
8. Add `.env.local.example` documenting required environment variables (API base URLs for Equipment Catalog and Morgan APIs).
9. Resolve dp-11 (navigation paradigm) by establishing the shell layout component (top nav bar or sidebar) in `app/layout.tsx`.

## Validation
Run `npm run dev` and confirm the app starts without errors at localhost:3000. Verify TailwindCSS utility classes render correctly. Verify Effect can be imported and a trivial Effect.succeed() runs. Confirm TypeScript compilation passes with `npm run build`.