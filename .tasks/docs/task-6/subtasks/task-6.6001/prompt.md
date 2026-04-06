Implement subtask 6001: Scaffold Elysia/Effect project with PostgreSQL models and migrations

## Objective
Initialize the Social Media Engine Node.js project with Elysia 1.x, Effect 3.x, and PostgreSQL integration. Define draft and published post data models and create database migrations.

## Steps
1. Initialize project with `bun init` and add dependencies: elysia, @elysiajs/cors, effect, @effect/schema, drizzle-orm (or kysely) with pg driver.
2. Configure TypeScript with strict mode.
3. Define database models:
   - `drafts` table: id (UUID), title, image_urls (text[]), caption, platform_targets (text[]), ai_curation_metadata (JSONB), status (enum: pending_review, approved, rejected, published), created_at, updated_at.
   - `published_posts` table: id (UUID), draft_id (FK), platform, platform_post_id, published_at, engagement_metrics (JSONB).
4. Write migrations for both tables with indexes on status and created_at.
5. Set up database connection pool reading DATABASE_URL from sigma1-infra-endpoints ConfigMap via envFrom.
6. Define Effect.Schema types mirroring the DB models for runtime validation.
7. Create Elysia app skeleton with health endpoint GET /healthz checking DB connectivity.
8. Add structured logging setup.

## Validation
Project builds and starts without errors; migrations run successfully against test PostgreSQL; health endpoint returns 200 with DB status; Effect.Schema types encode/decode correctly in unit tests.