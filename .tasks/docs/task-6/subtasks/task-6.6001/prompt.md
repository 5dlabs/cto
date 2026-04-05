Implement subtask 6001: Initialize Elysia/Effect project with PostgreSQL and S3/R2 connectivity

## Objective
Scaffold the Node.js service with Elysia 1.x and Effect 3.x, configure PostgreSQL connection pool and S3-compatible object storage client using infra ConfigMap values.

## Steps
1. Initialize a new Node.js 20+ project with Bun as runtime. Add dependencies: elysia 1.x, effect 3.x, @effect/schema, pg (or postgres.js), @aws-sdk/client-s3, uuid. 2. Configure PostgreSQL connection using DATABASE_URL from the infra ConfigMap (envFrom). Set up a connection pool with configurable pool size. 3. Configure S3 client pointing to the chosen object storage (R2 or S3) using S3_ENDPOINT, S3_ACCESS_KEY, S3_SECRET_KEY, S3_BUCKET from ConfigMap/secrets. 4. Create database migrations for social media tables: drafts (id, image_urls JSONB, caption, ai_score, status enum [draft/approved/rejected/published], platform_targets JSONB, created_at, updated_at), published_posts (id, draft_id FK, platform, platform_post_id, published_at, url). 5. Run migrations and verify connectivity. 6. Add GET /healthz endpoint checking both DB and S3 connectivity. 7. Set up the Elysia app skeleton with Effect layers for DB and S3 services.

## Validation
Verify migrations create correct tables; health endpoint returns 200 when DB and S3 are reachable; S3 client can list bucket contents; DB pool connects and queries successfully.