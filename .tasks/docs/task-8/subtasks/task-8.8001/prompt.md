Implement subtask 8001: Initialize Next.js 15 project with App Router, TypeScript, and TailwindCSS 4

## Objective
Scaffold the Next.js 15 project with App Router, TypeScript 5.x strict mode, and TailwindCSS 4. Configure project structure with app directory, layout files, global styles, and environment variable handling for API endpoints and CDN URLs.

## Steps
1. Run `npx create-next-app@latest` with App Router, TypeScript, TailwindCSS options.
2. Configure `tsconfig.json` with strict mode, path aliases (`@/components`, `@/lib`, `@/hooks`, `@/types`).
3. Set up TailwindCSS 4 with `@import "tailwindcss"` in global CSS.
4. Create root layout (`app/layout.tsx`) with html lang, viewport meta, dark theme class on body.
5. Set up `.env.local` with `NEXT_PUBLIC_API_BASE_URL`, `NEXT_PUBLIC_CDN_URL`, `NEXT_PUBLIC_WS_URL` placeholders.
6. Create folder structure: `app/`, `components/ui/`, `components/custom/`, `lib/`, `hooks/`, `types/`, `public/`.
7. Add `next.config.ts` with image remote patterns for R2 CDN domain.
8. Verify `npm run dev` starts without errors and renders a placeholder home page.

## Validation
Run `npm run dev` and verify dev server starts. Run `npm run build` and verify it completes without TypeScript or build errors. Confirm TailwindCSS utility classes render correctly on a test element.