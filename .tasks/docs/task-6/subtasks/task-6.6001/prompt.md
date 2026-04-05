Implement subtask 6001: Initialize Bun/Elysia/Effect project with TypeScript configuration

## Objective
Scaffold the Social Media Engine project with Bun runtime, Elysia web framework, Effect 3.x, TypeScript 5.x strict mode, and biome.js for linting/formatting. Set up the project structure with standard directories for routes, services, migrations, and tests.

## Steps
1. Create project directory `social-engine/` and run `bun init`.
2. Install core dependencies: `elysia@^1.0`, `effect@^3.0`, `typescript@^5.0`, `@elysiajs/cors`, `@elysiajs/bearer`.
3. Install dev dependencies: `@biomejs/biome`, `@types/bun`.
4. Configure `tsconfig.json` with `strict: true`, `moduleResolution: bundler`, `target: ES2022`, and Effect plugin if needed.
5. Configure `biome.json` with formatting and linting rules.
6. Create directory structure: `src/routes/`, `src/services/`, `src/db/`, `src/middleware/`, `src/schemas/`, `src/lib/`, `tests/`.
7. Create `src/index.ts` entry point with basic Elysia app that starts on configurable port (default 3000).
8. Add `scripts` in `package.json`: `dev`, `build`, `start`, `test`, `lint`, `migrate`.
9. Verify the app starts with `bun run dev` and responds on root route.

## Validation
Run `bun run dev` and confirm the server starts without errors. Run `bun run lint` and confirm biome passes. Verify TypeScript compilation succeeds with `bun run build`.