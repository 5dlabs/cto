Implement subtask 6002: Define Effect.Schema models and database migrations for social media entities

## Objective
Create Effect.Schema definitions for all domain entities (Upload, Draft, PublishedPost) and write PostgreSQL migrations for the corresponding tables.

## Steps
1. In `src/schemas/upload.ts`, define Upload schema: id (UUID), file_key (S3 key), original_filename, mime_type, file_size, uploaded_at, metadata (JSON). 2. In `src/schemas/draft.ts`, define Draft schema: id (UUID), upload_ids (UUID[]), caption (string), ai_generated_caption (string), platform_targets (array of INSTAGRAM/LINKEDIN/TIKTOK/FACEBOOK), status (enum: PENDING_REVIEW/APPROVED/REJECTED/PUBLISHED), reviewer_notes (optional string), created_at, updated_at. 3. In `src/schemas/published_post.ts`, define PublishedPost schema: id (UUID), draft_id (UUID), platform, platform_post_id, published_at, post_url. 4. Create request/response schemas for each endpoint using Effect.Schema. 5. Write SQL migrations: CREATE TABLE uploads, drafts, published_posts with proper indexes and foreign keys. 6. Add enum types for draft_status and platform.

## Validation
Migrations run successfully; Effect.Schema encode/decode roundtrips work for all entities; validation rejects invalid data (e.g., empty caption, invalid platform); schema types are correctly inferred by TypeScript.