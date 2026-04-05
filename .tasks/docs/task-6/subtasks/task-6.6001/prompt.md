Implement subtask 6001: Initialize Bun/Elysia/Effect project scaffold and database migrations for social schema

## Objective
Set up the project structure with Bun runtime, Elysia 1.x, Effect 3.x, and TypeScript 5.x. Create database migrations for the social schema including uploads, photos, drafts, and published_posts tables. Configure the Postgres client library and connection pooling via Effect layers.

## Steps
1. Initialize project: `bun init`, install elysia@1.x, effect@3.x, typescript@5.x, and chosen Postgres client library.
2. Set up tsconfig.json with strict mode, ES2022 target, and Effect-compatible settings.
3. Create `src/` directory structure: `src/routes/`, `src/services/`, `src/db/`, `src/pipelines/`, `src/lib/`.
4. Create migration files in `src/db/migrations/`:
   - Migration 001: `CREATE SCHEMA IF NOT EXISTS social;`
   - Migration 002: `uploads` table — id (UUID PK DEFAULT gen_random_uuid()), event_name (TEXT NOT NULL), uploaded_by (TEXT NOT NULL), uploaded_at (TIMESTAMPTZ DEFAULT now()), photo_count (INT NOT NULL DEFAULT 0).
   - Migration 003: `photos` table — id (UUID PK), upload_id (UUID FK → uploads.id ON DELETE CASCADE), r2_key (TEXT NOT NULL UNIQUE), original_filename (TEXT), width (INT), height (INT), ai_score (FLOAT), selected (BOOL DEFAULT false). Index on upload_id, index on ai_score.
   - Migration 004: `drafts` table — id (UUID PK), upload_id (UUID FK → uploads.id), platform (TEXT CHECK IN ('instagram','linkedin','facebook','tiktok')), caption (TEXT), hashtags (TEXT[]), image_keys (TEXT[]), status (TEXT CHECK IN ('draft','approved','rejected','published','failed') DEFAULT 'draft'), approved_by (TEXT), approved_at (TIMESTAMPTZ), published_at (TIMESTAMPTZ), platform_post_id (TEXT). Index on status, index on upload_id.
   - Migration 005: `published_posts` table — id (UUID PK), draft_id (UUID FK → drafts.id), platform (TEXT), post_url (TEXT), engagement_data (JSONB DEFAULT '{}'), published_at (TIMESTAMPTZ DEFAULT now()). Index on draft_id.
5. Create `src/db/client.ts` — Effect Layer for Postgres connection using envFrom ConfigMap values (DB_HOST, DB_PORT, DB_NAME, DB_USER, DB_PASSWORD).
6. Create a migration runner script `src/db/migrate.ts` that applies migrations in order.
7. Add `bun run migrate` script to package.json.

## Validation
Run migrations against a test Postgres instance (via testcontainers or local Docker). Verify all 4 tables exist in the social schema with correct columns, types, constraints, and indexes. Verify foreign key cascades by inserting an upload, then deleting it and confirming photos/drafts are removed.