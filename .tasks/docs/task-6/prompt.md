<identity>
You are nova, the Node.js 20+/Elysia 1.x/Effect 3.x implementation agent. You own task 6 end-to-end.
</identity>

<context>
<task_overview>
Task 6: Build Social Media Engine (Nova - Node.js/Elysia+Effect)
Implement the Social Media Engine using Node.js 20, Elysia 1.x, and Effect 3.x TypeScript. Handles event photo ingestion, AI-powered image curation and scoring, platform-specific cropping, caption generation via OpenAI/Claude, draft approval workflow via Signal, and multi-platform publishing to Instagram, LinkedIn, TikTok, and Facebook with Effect retry and backoff. Includes internal GDPR endpoints.
Priority: medium
Dependencies: 1
</task_overview>
</context>

<implementation_plan>
1. Initialize project at services/social-engine using Bun or Node.js 20 with TypeScript 5.x. Install: elysia@1.x, @elysiajs/swagger, effect@3.x, @aws-sdk/client-s3, pg (postgres.js or @vercel/postgres), ioredis, sharp (image processing), openai, @anthropic-ai/sdk, zod (for any non-Effect validation), pino for logging, prom-client.
2. Database migrations targeting social schema. Tables: social_drafts (id UUID PK, event_id UUID, source_photo_urls TEXT[], selected_photo_urls TEXT[], instagram_crop_url TEXT, linkedin_crop_url TEXT, tiktok_crop_url TEXT, caption TEXT, hashtags TEXT[], status TEXT CHECK IN (pending_curation,ready_for_approval,approved,rejected,published), approved_by TEXT, approved_at TIMESTAMPTZ, created_at TIMESTAMPTZ), social_posts (id UUID PK, draft_id UUID FK, platform TEXT CHECK IN (instagram,linkedin,tiktok,facebook), external_post_id TEXT, published_at TIMESTAMPTZ, status TEXT CHECK IN (pending,published,failed), error_text TEXT).
3. Effect.Service definitions: InstagramService (publishPost, uploadMedia), LinkedInService (publishPost), TikTokService (uploadVideo), FacebookService (publishPost). Each service uses Effect.retry with exponential backoff (Schedule.exponential(1000ms) with max 5 attempts) and Effect.timeout(30s).
4. Effect.Schema definitions for all request/response types: UploadEventPhotosRequest, DraftResponse, PublishRequest.
5. Elysia routes: POST /api/v1/social/upload (multipart, upload photos to R2, create draft with status=pending_curation, trigger curation pipeline), GET /api/v1/social/drafts, GET /api/v1/social/drafts/:id, POST /api/v1/social/drafts/:id/approve, POST /api/v1/social/drafts/:id/reject, POST /api/v1/social/drafts/:id/publish, GET /api/v1/social/published.
6. AI curation pipeline (triggered after upload): Call OpenAI vision API (gpt-4o) scoring each photo on composition (0-10), lighting (0-10), subject clarity (0-10). Select top 5-10 photos. Use sharp to generate crops: Instagram 1:1 (1080x1080), Instagram Story 9:16 (1080x1920), LinkedIn 1.91:1 (1200x628), TikTok 9:16 (1080x1920). Upload cropped versions to R2. Update draft with selected_photo_urls and crop URLs.
7. Caption generation: POST to OpenAI chat API with event context (event name, venue, equipment used from catalog service), generate caption + hashtags. Store in draft.caption and draft.hashtags.
8. Approval workflow: POST /api/v1/social/drafts/:id/approve sets status=ready_for_review (Morgan sends Signal message to configured phone number with draft preview link). POST approve sets status=approved. POST reject sets status=rejected with optional reason.
9. Publishing: on POST /api/v1/social/drafts/:id/publish, run Effect program calling each platform service. Instagram: use Graph API /media + /media_publish. LinkedIn: use Share API /ugcPosts. Facebook: use Graph API /photos or /feed. TikTok: use Content Posting API. On each success, insert social_posts row. Portfolio sync: published posts exposed via GET /api/v1/social/published for website portfolio page.
10. JWT middleware via Elysia plugin, Prometheus metrics endpoint /metrics using prom-client, /health/live, /health/ready (postgres + redis + R2 connectivity). GDPR: GET /internal/gdpr/export/:customer_id, DELETE /internal/gdpr/delete/:customer_id (anonymize event_id references). Kubernetes Deployment 2 replicas.
</implementation_plan>

<acceptance_criteria>
1. POST /api/v1/social/upload with 3 test JPEG files returns 202 with draft_id and status=pending_curation.
2. After curation pipeline completes (poll GET /api/v1/social/drafts/:id), status=ready_for_approval and selected_photo_urls contains at least 1 URL pointing to R2 bucket.
3. POST /api/v1/social/drafts/:id/approve returns 200 and status transitions to approved.
4. POST /api/v1/social/drafts/:id/publish with approved draft returns 200; GET /api/v1/social/published includes the new post with platform=instagram and status=published.
5. Effect retry: mock Instagram API to fail twice then succeed; verify retry count = 2 in logs and final status=published.
6. GET /metrics returns prom-client text format with http_requests_total counter.
7. GET /health/ready returns 200 with all dependencies healthy.
8. vitest or jest test suite passes with >= 80% coverage including Effect service unit tests.

See also: acceptance.md in this task directory for the checklist version.
</acceptance_criteria>

<subtasks>
- Initialize Node.js/TypeScript project and install all dependencies: Scaffold the services/social-engine project with TypeScript 5.x configuration, install all required packages, and configure tsconfig, pino logging, and the Elysia app entry point.
- Write database migrations for social schema tables: Create SQL migration files for the social schema: social_drafts and social_posts tables with all columns, constraints, foreign keys, and indexes.
- Define Effect.Schema types and Effect.Service interfaces: Define all Effect.Schema request/response types (UploadEventPhotosRequest, DraftResponse, PublishRequest) and the Effect.Service interfaces for InstagramService, LinkedInService, TikTokService, and FacebookService with retry and timeout configuration.
- Implement photo upload handler and R2 multipart storage: Implement POST /api/v1/social/upload as a multipart Elysia route that accepts JPEG/PNG files, uploads them to Cloudflare R2 via the S3-compatible SDK, creates a social_drafts row with status=pending_curation, and triggers the async curation pipeline.
- Implement AI photo curation pipeline with OpenAI vision scoring: Implement the async curation pipeline that scores each uploaded photo using OpenAI gpt-4o vision API, selects the top 5-10 photos, and updates the draft's selected_photo_urls.
- Implement sharp image cropping for platform-specific formats and R2 upload: Implement the image cropping step within the curation pipeline: use sharp to generate Instagram 1:1, Instagram Story 9:16, LinkedIn 1.91:1, and TikTok 9:16 crops for each selected photo, then upload cropped versions to R2.
- Implement caption generation via OpenAI chat API: Implement the caption generation step that assembles event context and calls the OpenAI chat API to generate a caption and hashtags, storing results on the social_drafts row.
- Implement approval workflow endpoints and Signal notification: Implement POST /api/v1/social/drafts/:id/approve, POST /api/v1/social/drafts/:id/reject, and the Signal notification trigger that sends a preview link to the configured phone number when a draft reaches ready_for_approval.
- Implement InstagramService with Effect retry and Graph API integration: Implement the InstagramService Effect.Service that uploads media and publishes to Instagram via the Graph API, with exponential backoff retry (max 5 attempts) and 30-second timeout per call.
- Implement LinkedInService with Effect retry and Share API integration: Implement the LinkedInService Effect.Service that publishes posts to LinkedIn via the UGC Posts Share API with exponential backoff retry and timeout.
- Implement TikTokService with Effect retry and Content Posting API integration: Implement the TikTokService Effect.Service that uploads video/photo content to TikTok via the Content Posting API with exponential backoff retry and timeout.
- Implement FacebookService with Effect retry and Graph API integration: Implement the FacebookService Effect.Service that publishes posts to Facebook via the Graph API /photos or /feed endpoint with exponential backoff retry and timeout.
- Implement POST /api/v1/social/drafts/:id/publish orchestration and GET /api/v1/social/published: Implement the publish endpoint that runs all four platform Effect services in parallel for an approved draft, records per-platform social_posts rows, and exposes GET /api/v1/social/published for portfolio sync.
- Implement JWT middleware, health endpoints, Prometheus metrics, and GDPR endpoints: Wire up JWT authentication middleware on all /api/v1/* routes, implement /health/live, /health/ready (postgres + redis + R2), /metrics with prom-client, GDPR export/delete internal endpoints, and the Kubernetes Deployment manifest.
- Write vitest test suite targeting >= 80% coverage: Write the complete vitest test suite covering upload handler, curation pipeline, caption generation, approval workflow, all four platform Effect services with retry behavior, publish orchestration, and GDPR endpoints.
</subtasks>