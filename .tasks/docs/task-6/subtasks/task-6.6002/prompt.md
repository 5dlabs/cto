Implement subtask 6002: Implement PostgreSQL connection layer with Effect

## Objective
Create an Effect Layer for PostgreSQL connectivity using the connection string from the ConfigMap. Implement the database client service, connection pool, and run the social media schema migrations.

## Steps
1. Install `postgres` (postgres.js) or `@effect/sql-pg` for PostgreSQL connectivity. 2. Create `src/layers/DatabaseLayer.ts` as an Effect.Layer that reads DATABASE_URL from env (provided by ConfigMap). 3. Create `src/services/DatabaseService.ts` as an Effect.Service tag with query, transaction, and pool-status methods. 4. Create migration files in `migrations/` directory for social media tables: `social_photos` (id, s3_key, original_url, thumbnail_url, metadata jsonb, uploaded_at), `social_drafts` (id, photo_id, caption, platform, status enum [draft,pending_approval,approved,rejected,published], ai_model_used, created_at, updated_at), `social_published` (id, draft_id, platform, platform_post_id, published_at, engagement_metrics jsonb), `social_platform_credentials` (id, platform, access_token_encrypted, refresh_token_encrypted, expires_at). 5. Implement a simple migration runner that executes SQL files in order. 6. Add connection health check to the health endpoint.

## Validation
Migration runner executes without errors against a test PostgreSQL instance. DatabaseService can insert and query from social_photos and social_drafts. Connection pool health check returns connected status.