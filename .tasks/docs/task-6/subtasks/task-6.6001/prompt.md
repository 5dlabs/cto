Implement subtask 6001: Scaffold Elysia project with Effect 3.x and TypeScript 5.x

## Objective
Initialize the Social Media Engine project with Elysia 1.x, Effect 3.x, and TypeScript 5.x. Set up the project structure, tsconfig, package.json with Bun as runtime, and Effect layer/service architecture. Include health check endpoint at /api/v1/social/health and Prometheus metrics endpoint.

## Steps
1. Create project directory `services/social-media-engine`. 2. Initialize with `bun init` and install dependencies: elysia, @elysiajs/swagger, effect, @effect/schema, @effect/platform, prom-client. 3. Configure tsconfig.json with strict mode, ESM modules, paths aliases. 4. Create project structure: src/index.ts (Elysia app entry), src/layers/ (Effect layers), src/services/ (Effect services), src/routes/ (Elysia route modules), src/schemas/ (Effect.Schema definitions), src/errors/ (tagged errors). 5. Implement MainLayer as the root Effect Layer that composes all service layers. 6. Add GET /api/v1/social/health returning {status: 'ok', timestamp}. 7. Add GET /metrics endpoint exposing Prometheus counters. 8. Add Dockerfile for Bun runtime with multi-stage build. 9. Read ConfigMap endpoints from environment variables via `envFrom` referencing `{project}-infra-endpoints`.

## Validation
Run `bun run src/index.ts` and verify health endpoint returns 200 with JSON body. Verify /metrics returns Prometheus text format. TypeScript compiles with zero errors under strict mode. Dockerfile builds successfully.