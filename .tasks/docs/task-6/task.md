## Develop Social Media Engine (Nova - Node.js/Elysia+Effect)

### Objective
Build the Social Media Engine for automated content curation, AI caption generation, multi-platform publishing (Instagram, LinkedIn, Facebook, TikTok), and Signal-based approval workflow. Uses NATS for async publish pipeline per D4 resolution.

### Ownership
- Agent: nova
- Stack: Node.js 20+/Elysia 1.x + Effect 3.x
- Priority: medium
- Status: pending
- Dependencies: 1

### Implementation Details
1. Initialize project with Bun runtime, Elysia 1.x framework, Effect 3.x, TypeScript 5.x.
2. Database: Use `@effect/sql-pg` or `postgres` (postgresjs) library. Migrations for `social` schema:
   - `uploads` table: id, event_name, uploaded_by, uploaded_at, photo_count.
   - `photos` table: id, upload_id, r2_key, original_filename, width, height, ai_score (FLOAT), selected (BOOL).
   - `drafts` table: id, upload_id, platform (instagram/linkedin/facebook/tiktok), caption, hashtags (TEXT[]), image_keys (TEXT[]), status (draft/approved/rejected/published/failed), approved_by, approved_at, published_at, platform_post_id.
   - `published_posts` table: id, draft_id, platform, post_url, engagement_data (JSONB), published_at.
3. Implement REST endpoints with Elysia + Effect Schema validation:
   - `POST /api/v1/social/upload` — accepts multipart form with photos. Upload to R2, store metadata, trigger AI curation pipeline.
   - `GET /api/v1/social/drafts` — list drafts with pagination and status filter.
   - `GET /api/v1/social/drafts/:id` — draft detail with photos and caption.
   - `POST /api/v1/social/drafts/:id/approve` — mark approved, publish to NATS `social.publish` subject.
   - `POST /api/v1/social/drafts/:id/reject` — mark rejected with optional reason.
   - `POST /api/v1/social/drafts/:id/publish` — manual publish trigger (also publishes to NATS).
   - `GET /api/v1/social/published` — list published posts.
4. AI Curation Pipeline (triggered after upload):
   - Use OpenAI Vision API (or Claude) to score each photo for composition, lighting quality, brand relevance.
   - Select top 5-10 images based on score threshold.
   - Generate platform-specific crops: Instagram square (1:1) + Story (9:16), LinkedIn landscape (1.91:1), TikTok (9:16).
   - Use `sharp` library for image processing.
   - Generate captions using OpenAI/Claude with event context, equipment featured, and platform-specific tone.
   - Create draft records for each platform with appropriate crop + caption.
5. Effect Service pattern:
   - `InstagramService` — Instagram Graph API client (Effect.Service)
   - `LinkedInService` — LinkedIn API client (Effect.Service)
   - `FacebookService` — Facebook Graph API client (Effect.Service)
   - `TikTokService` — TikTok API client (Effect.Service)
   - All use `Effect.retry` with exponential backoff for API delivery.
6. NATS integration (per D4 — async only for social publish):
   - Subscribe to `social.publish` subject using `nats` npm package.
   - On message: call appropriate platform service to publish, update draft status, create published_posts record.
   - Dead-letter handling: after 3 failed attempts, mark draft as `failed` with error.
7. R2 integration: Upload photos to Cloudflare R2 via S3-compatible API (`@aws-sdk/client-s3`). Generate CDN URLs for published content.
8. Health/metrics: `/health/live`, `/health/ready`, `/metrics` (prom-client).
9. GDPR endpoint: `DELETE /api/v1/gdpr/customer/:id` — delete photos and drafts associated with customer events, remove from R2, return confirmation.
10. OpenAPI spec generation via Elysia's built-in Swagger plugin.
11. Dockerfile: Bun-based image. Kubernetes Deployment: namespace sigma1, replicas 1 (medium priority), envFrom ConfigMap, secrets for R2, OpenAI API key, platform API credentials, NATS URL.

### Subtasks
- [ ] Initialize Bun/Elysia/Effect project scaffold and database migrations for social schema: Set up the project structure with Bun runtime, Elysia 1.x, Effect 3.x, and TypeScript 5.x. Create database migrations for the social schema including uploads, photos, drafts, and published_posts tables. Configure the Postgres client library and connection pooling via Effect layers.
- [ ] Implement Cloudflare R2 integration service with S3-compatible client: Build an Effect Service for Cloudflare R2 using @aws-sdk/client-s3 that handles photo uploads, deletion, and CDN URL generation. This service is used by the upload endpoint, AI pipeline, and GDPR deletion.
- [ ] Implement photo upload endpoint with multipart handling and R2 storage: Build the POST /api/v1/social/upload endpoint that accepts multipart form data with photos, uploads them to R2, extracts image dimensions, and stores metadata in the uploads and photos tables.
- [ ] Implement draft management REST endpoints with Effect Schema validation: Build the draft listing, detail, approve, reject, and publish endpoints. Also implement the published posts listing endpoint. All endpoints use Effect Schema for request/response validation.
- [ ] Implement AI photo scoring pipeline with OpenAI Vision API: Build the AI photo scoring service that uses OpenAI Vision API to evaluate each uploaded photo for composition, lighting quality, and brand relevance, then selects the top photos based on score threshold.
- [ ] Implement platform-specific image cropping pipeline with sharp: Build the image processing service that generates platform-specific crops from selected photos: Instagram square (1:1) and Story (9:16), LinkedIn landscape (1.91:1), Facebook (1.91:1), and TikTok (9:16).
- [ ] Implement AI caption generation and draft creation pipeline: Build the caption generation service using OpenAI/Claude that creates platform-specific captions with appropriate tone, hashtags, and formatting. Wire the full AI curation pipeline (score → select → crop → caption → create drafts) triggered after upload.
- [ ] Implement Effect Service clients for Instagram, LinkedIn, Facebook, and TikTok APIs: Build four separate Effect.Service implementations for publishing content to each social media platform via their respective APIs, all with exponential backoff retry logic.
- [ ] Implement NATS integration for async publish pipeline with dead-letter handling: Build the NATS subscriber that listens to the social.publish subject, dispatches to platform services, updates draft/published_posts records, and handles dead-letter after 3 failed attempts. Wire the approve and publish endpoints to emit NATS messages.
- [ ] Implement GDPR deletion endpoint with R2 cleanup: Build the DELETE /api/v1/gdpr/customer/:id endpoint that removes all photos, drafts, published posts, and R2 objects associated with a customer's events.
- [ ] Implement health/metrics endpoints, OpenAPI spec, Dockerfile, and Kubernetes deployment manifest: Add health check endpoints, Prometheus metrics, OpenAPI documentation via Elysia Swagger plugin, Bun-based Dockerfile, and Kubernetes deployment manifest for the sigma1 namespace.