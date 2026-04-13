<identity>
You are blaze working on subtask 8001 of task 8.
</identity>

<context>
<scope>
Scaffold the Next.js 15 App Router project with all required dependencies installed and baseline configuration files in place.
</scope>
</context>

<implementation_plan>
1. From monorepo root: `npx create-next-app@15 apps/website --typescript --tailwind --app --src-dir false --import-alias '@/*'`.
2. Install additional dependencies: `cd apps/website && bun add effect@3.x @effect/schema @tanstack/react-query@5 react-day-picker @cloudflare/next-on-pages`.
3. Install dev dependencies: `bun add -d @playwright/test lighthouse`.
4. Update next.config.ts: add `experimental: { reactCompiler: true }` for React 19 compiler support. Add image domains for R2 CDN hostname.
5. Create .env.local.example with: NEXT_PUBLIC_API_URL=, NEXT_PUBLIC_MORGAN_WIDGET_URL=.
6. Verify `bun run build` exits 0 on the empty scaffold.
7. Add apps/website to monorepo workspace in root package.json (if using bun workspaces).
</implementation_plan>

<validation>
`cd apps/website && bun run build` completes without errors. `bun run dev` starts server on port 3000 and GET http://localhost:3000 returns HTTP 200. TypeScript version >= 5.4 confirmed via `tsc --version`.
</validation>