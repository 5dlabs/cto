Implement subtask 6001: Initialize Elysia project with Effect 3.x, PostgreSQL, and S3/R2 connectivity

## Objective
Scaffold the Node.js Elysia project with Effect 3.x integration, establish PostgreSQL connection pool and S3/R2 object storage client, reading all configuration from the 'sigma1-infra-endpoints' ConfigMap.

## Steps
1. Initialize project with `bun init`, add dependencies: elysia, effect (3.x), @effect/schema, @effect/sql, @effect/sql-pg, @aws-sdk/client-s3 (for R2/S3 compatibility). 2. Create project structure: src/config/, src/routes/, src/services/, src/models/, src/integrations/, src/schemas/. 3. In `src/config/index.ts`, read DATABASE_URL, S3_ENDPOINT, S3_BUCKET, S3_ACCESS_KEY, S3_SECRET_KEY, OPENAI_API_KEY (or ANTHROPIC_API_KEY), and platform API keys from environment (sourced from sigma1-infra-endpoints). 4. Create `src/services/database.ts` — an Effect.Layer providing a PostgreSQL connection pool via @effect/sql-pg. 5. Create `src/services/storage.ts` — an Effect.Layer providing an S3Client configured for R2/S3 with upload/download/delete methods. 6. Set up the Elysia app in `src/index.ts` with health check endpoint. 7. Create a Dockerfile. 8. Run database migrations for social media tables (drafts, published_posts, uploads).

## Validation
Service starts on configured port; GET /health returns 200; PostgreSQL connection pool initializes; S3 client connects to configured endpoint; all environment variables are loaded; `bun run build` succeeds.