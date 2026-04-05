Implement subtask 6001: Initialize Elysia project with Effect, PostgreSQL integration, and data models

## Objective
Set up the Node.js 20+ project with Elysia 1.x and Effect 3.x, define the database schema for social media drafts and published posts, and create the base Elysia router with Effect.Schema validation.

## Steps
1. Initialize the project with `bun init`. Add dependencies: elysia 1.x, effect 3.x, @effect/schema, drizzle-orm (with postgres driver), and pg. 2. Set up the Elysia application entrypoint with CORS, error handling, and logging middleware. 3. Define data models using Effect.Schema: a) `Draft` — id (UUID), event_name, photos (array of photo references), selected_photos (curated subset), caption, platform_targets (array of 'instagram'|'linkedin'|'facebook'), status (enum: 'pending_curation'|'pending_caption'|'pending_approval'|'approved'|'rejected'|'published'), created_at, updated_at. b) `PublishedPost` — id, draft_id, platform, platform_post_id, published_at, engagement_metrics (nullable JSON). 4. Create Drizzle ORM schema and migration files for `social_drafts` and `social_published_posts` tables. 5. Set up PostgreSQL connection pool reading from environment variables (infra-endpoints ConfigMap). 6. Create the Elysia router skeleton with all endpoint paths returning 501 placeholders. 7. Validate that Effect.Schema is wired into request/response validation on the upload endpoint as a proof of concept.

## Validation
Project builds and starts with `bun run`. Migrations run successfully against test PostgreSQL. Effect.Schema validation rejects malformed requests on the upload endpoint. All placeholder routes return 501. Drizzle ORM can insert and query a draft record.