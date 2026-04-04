Implement subtask 7001: Scaffold Next.js App Router project with TypeScript and Tailwind CSS

## Objective
Initialize a new Next.js application using the App Router with TypeScript enabled and Tailwind CSS configured. Set up the project structure with `src/app` directory layout, configure `tsconfig.json`, `tailwind.config.ts`, and `postcss.config.js`. Ensure `next build` passes cleanly with no errors.

## Steps
1. Run `npx create-next-app@latest` with `--typescript`, `--tailwind`, `--app`, `--src-dir` flags.
2. Verify `tsconfig.json` has strict mode enabled and path aliases configured (`@/*` → `src/*`).
3. Confirm Tailwind CSS is working by adding a test utility class to `src/app/page.tsx` and verifying it renders.
4. Set up base layout in `src/app/layout.tsx` with html lang attribute and basic metadata.
5. Add `PM_SERVER_URL` to `.env.local` and `next.config.js` for runtime environment variable exposure.
6. Verify `next dev` starts and `next build` exits 0.

## Validation
`next build` exits 0. `next dev` starts on port 3000 and renders the default page. Tailwind utility classes render correctly in the browser.