Implement subtask 1001: Initialize Next.js 14 App Router project with TypeScript, Tailwind CSS, and linting

## Objective
Scaffold the Next.js 14 project using App Router, configure TypeScript strict mode, install and configure Tailwind CSS 3.4+, and set up ESLint and Prettier with consistent rules. This is the foundation that all other subtasks depend on.

## Steps
1. Run `npx create-next-app@latest --typescript --app --tailwind` to scaffold the project.
2. Enable `strict: true` in `tsconfig.json`.
3. Verify Tailwind CSS 3.4+ is installed; update `tailwind.config.ts` with `content` paths covering `app/` and `components/`.
4. Install and configure ESLint with `eslint-config-next` and Prettier. Add `.prettierrc` with consistent formatting rules (`semi: true`, `singleQuote: true`, `trailingComma: 'all'`).
5. Add `lint` and `format` scripts to `package.json`.
6. Install Playwright and `@axe-core/playwright` as dev dependencies now so they're available for later subtasks: `npm i -D @playwright/test @axe-core/playwright`.
7. Run `npx playwright install --with-deps chromium`.
8. Run `next build` to confirm zero errors on the clean scaffold.
9. Commit the initial project structure.

## Validation
`next build` completes with zero errors. `npm run lint` passes with no warnings or errors. `npx tsc --noEmit` reports zero type errors. Tailwind utility classes render correctly when adding a test `<div className="bg-blue-500">` to the default page. Playwright is importable: `npx playwright test --list` runs without error.