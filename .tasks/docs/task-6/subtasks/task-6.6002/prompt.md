Implement subtask 6002: Create database migrations for social schema (uploads, drafts, published_posts)

## Objective
Implement database migrations using drizzle-orm (or kysely) for the `social` schema, defining the uploads, drafts, and published_posts tables with all columns, types, indexes, and foreign key constraints as specified.

## Steps
1. Install `drizzle-orm`, `drizzle-kit`, `postgres` (pg driver for Bun).
2. Configure `drizzle.config.ts` pointing to `POSTGRES_URL` from environment (sourced from sigma1-infra-endpoints ConfigMap).
3. Create schema file `src/db/schema.ts`:
   - Define `social` schema namespace.
   - `uploads` table: `id` (UUID PK, default gen_random_uuid()), `event_id` (UUID, nullable), `original_url` (TEXT NOT NULL — R2 key), `thumbnail_url` (TEXT nullable), `metadata` (JSONB: exif, dimensions), `uploaded_at` (TIMESTAMPTZ, default now()).
   - `drafts` table: `id` (UUID PK), `upload_ids` (UUID[] NOT NULL), `caption` (TEXT), `hashtags` (TEXT[]), `platforms` (TEXT[] NOT NULL), `status` (TEXT NOT NULL, default 'draft', CHECK in draft/pending_approval/approved/rejected/published/failed), `platform_crops` (JSONB), `ai_score` (REAL), `created_at` (TIMESTAMPTZ, default now()), `updated_at` (TIMESTAMPTZ, default now()).
   - `published_posts` table: `id` (UUID PK), `draft_id` (UUID FK → drafts.id ON DELETE CASCADE), `platform` (TEXT NOT NULL), `platform_post_id` (TEXT), `published_at` (TIMESTAMPTZ, default now()), `engagement_data` (JSONB nullable).
4. Add indexes: `drafts(status)`, `drafts(created_at)`, `published_posts(platform, published_at)`.
5. Create `src/db/client.ts` that exports a configured drizzle instance.
6. Generate and run migrations with `drizzle-kit generate` and `drizzle-kit push`.
7. Add a migration script in package.json.

## Validation
Run the migration against a test PostgreSQL instance. Verify all three tables exist in the `social` schema with correct columns, types, and indexes via `\d social.uploads`, `\d social.drafts`, `\d social.published_posts`. Insert and query a sample row in each table.