Implement subtask 6001: Initialize Elysia/Effect project with database schema and validation setup

## Objective
Scaffold the Node.js 20+ project with Elysia 1.x and Effect 3.x, configure PostgreSQL and S3 connectivity from ConfigMap, create database migrations for social media models, and set up Effect.Schema validation.

## Steps
1. Initialize a new Node.js project with Bun as the runtime. Install Elysia 1.x, Effect 3.x, @effect/schema, drizzle-orm (or kysely) with PostgreSQL driver, and @aws-sdk/client-s3.
2. Configure environment variable loading from POSTGRES_URL and S3_URL (via ConfigMap `envFrom`).
3. Create database migration files for the `social_media` schema with tables:
   - `photos` (id, filename, s3_key, s3_url, upload_date, metadata JSONB, curation_score, is_curated)
   - `drafts` (id, photo_ids JSONB array, caption_text, platform enum, ai_model_used, status enum: draft/pending_approval/approved/rejected, created_at, updated_at)
   - `approvals` (id, draft_id FK, approved_by, decision enum, signal_message_id, decided_at)
   - `published_posts` (id, draft_id FK, platform, external_post_id, published_at, sync_status)
4. Set up Effect.Schema definitions for all request/response types (PhotoUploadRequest, DraftCreateRequest, ApprovalRequest, PublishRequest, etc.).
5. Configure the Elysia app skeleton with error handling, CORS, and the base router.
6. Set up the S3 client for object storage operations.
7. Run migrations on startup.

## Validation
Verify the project starts without errors with `bun run`. Confirm migrations create all tables in a test PostgreSQL instance. Verify S3 client connects successfully. Verify Effect.Schema validators reject invalid input.