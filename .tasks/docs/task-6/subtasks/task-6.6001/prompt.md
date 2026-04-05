Implement subtask 6001: Scaffold Elysia/Effect TypeScript project with infrastructure wiring

## Objective
Initialize the Node.js 20+ project with Elysia 1.x, Effect TypeScript, and all necessary dependencies. Configure the project structure, environment loading from 'sigma1-infra-endpoints' ConfigMap, database connection via Effect layers, and a health check endpoint.

## Steps
1. Initialize project with `bun init` (or npm init) targeting Node.js 20+.
2. Install dependencies: elysia, @elysiajs/cors, effect, @effect/schema, @effect/platform, pg (or postgres.js), sharp (for image processing), and dev deps (vitest, typescript, @types/*).
3. Set up tsconfig.json with strict mode, Effect-compatible settings.
4. Create src/index.ts with Elysia app skeleton and GET /health endpoint.
5. Create src/config.ts using Effect.Config to read env vars: DATABASE_URL, S3_ENDPOINT, S3_BUCKET, S3_ACCESS_KEY, S3_SECRET_KEY, OPENAI_API_KEY, SIGNAL_WEBHOOK_URL, and per-platform social media API keys/tokens.
6. Create src/db.ts with Effect Layer for PostgreSQL connection pool.
7. Create project directory structure: src/services/, src/handlers/, src/models/, src/integrations/, src/pipelines/.
8. Verify the app starts and health endpoint returns 200.

## Validation
Project compiles with `bun build` or `tsc` without errors; health endpoint returns HTTP 200; config module correctly loads all expected environment variables; database layer can establish a connection.