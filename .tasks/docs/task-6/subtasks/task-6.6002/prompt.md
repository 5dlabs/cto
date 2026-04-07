Implement subtask 6002: Write database migrations for social schema tables

## Objective
Create SQL migration files for the social schema: social_drafts and social_posts tables with all columns, constraints, foreign keys, and indexes.

## Steps
Use a migration tool compatible with postgres.js (e.g., node-postgres-migrate or raw SQL files run at startup). Migration 001_social_drafts.sql: id UUID PK DEFAULT gen_random_uuid(), event_id UUID NOT NULL, source_photo_urls TEXT[] NOT NULL DEFAULT '{}', selected_photo_urls TEXT[] NOT NULL DEFAULT '{}', instagram_crop_url TEXT, linkedin_crop_url TEXT, tiktok_crop_url TEXT, caption TEXT, hashtags TEXT[] NOT NULL DEFAULT '{}', status TEXT NOT NULL CHECK (status IN ('pending_curation','ready_for_approval','approved','rejected','published')), approved_by TEXT, approved_at TIMESTAMPTZ, created_at TIMESTAMPTZ NOT NULL DEFAULT now(). Migration 002_social_posts.sql: id UUID PK DEFAULT gen_random_uuid(), draft_id UUID NOT NULL REFERENCES social.social_drafts(id), platform TEXT NOT NULL CHECK (platform IN ('instagram','linkedin','tiktok','facebook')), external_post_id TEXT, published_at TIMESTAMPTZ, status TEXT NOT NULL CHECK (status IN ('pending','published','failed')), error_text TEXT. Add index on social_drafts(event_id) and social_posts(draft_id). Run migrations at application startup before Elysia listens.

## Validation
Run migrations against a local Postgres instance; verify via `\d social.social_drafts` and `\d social.social_posts` that all columns exist with correct types. Run migrations twice and confirm idempotency. Insert a test draft and post row to confirm FK constraint works.