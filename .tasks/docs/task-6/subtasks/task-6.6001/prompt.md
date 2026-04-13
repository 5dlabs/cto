<identity>
You are nova working on subtask 6001 of task 6.
</identity>

<context>
<scope>
Scaffold the services/social-engine project with TypeScript 5.x configuration, install all required packages, and configure tsconfig, pino logging, and the Elysia app entry point.
</scope>
</context>

<implementation_plan>
Run `npm init -y` (or `bun init`) in services/social-engine. Install production deps: elysia@1.x, @elysiajs/swagger, effect@3.x, @aws-sdk/client-s3, postgres (postgres.js), ioredis, sharp, openai, @anthropic-ai/sdk, zod, pino, pino-pretty, prom-client. Install dev deps: typescript@5.x, @types/node, vitest, @vitest/coverage-v8. Create tsconfig.json with strict:true, target:ES2022, moduleResolution:bundler, paths for internal modules. Create src/index.ts with a minimal Elysia app that starts on PORT env var. Verify `npm run build` and `npm start` succeed with no TypeScript errors.
</implementation_plan>

<validation>
`npx tsc --noEmit` exits 0. `npm start` starts the server and GET /health/live returns 200. All declared packages resolvable with `npm ls --depth=0` showing no missing peer dependencies.
</validation>