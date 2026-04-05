Implement subtask 6002: Define data models and database migrations for social media drafts and published content

## Objective
Design and implement the data models for social media content lifecycle: uploads, drafts (with AI-generated captions), approval status, and published content records. Create PostgreSQL migrations for all tables.

## Steps
1. Create src/models/upload.ts: id (UUID), file_key (S3 key), original_filename, mime_type, width, height, file_size, uploaded_at.
2. Create src/models/draft.ts: id (UUID), upload_ids (UUID[]), ai_selected_images (UUID[]), caption (text), hashtags (text[]), platform_targets (enum[]: Instagram/LinkedIn/TikTok/Facebook), status (enum: Pending/Approved/Rejected/Published), ai_curation_metadata (JSONB), created_at, updated_at, approved_at, approved_by, rejection_reason.
3. Create src/models/published.ts: id (UUID), draft_id (UUID FK), platform (enum), platform_post_id (string), published_url (string), published_at, publish_status (enum: Success/Failed), error_message.
4. Create migrations with @effect/sql or raw SQL files for all tables in the appropriate schema.
5. Define @effect/schema schemas for request/response validation.
6. Add indexes on draft status, platform, and created_at for efficient querying.

## Validation
Migrations run successfully against test PostgreSQL; Effect schema validators correctly accept valid data and reject invalid data; models can be inserted and queried in integration tests.