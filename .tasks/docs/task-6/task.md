## Build Social Media Engine (Nova - Node.js/Elysia + Effect)

### Objective
Implement the Social Media Engine for automated content curation, AI-powered caption generation, multi-platform publishing (Instagram, LinkedIn, Facebook, TikTok), and Signal-based approval workflows. Uses Effect TypeScript for service composition, error handling, and retry logic.

### Ownership
- Agent: nova
- Stack: Node.js 20+/Elysia + Effect
- Priority: medium
- Status: pending
- Dependencies: 1

### Implementation Details
1. Initialize Node.js project with Bun runtime:
   - `package.json` with Elysia 1.x, Effect 3.x, TypeScript 5.x
   - biome.js for linting and formatting
   - `tsconfig.json` with strict mode and Effect plugin
2. Database migrations (drizzle-orm or kysely) in `social` schema:
   - `uploads` table: id (UUID PK), event_id (UUID), original_url (R2 key), thumbnail_url, metadata (JSONB: exif, dimensions), uploaded_at
   - `drafts` table: id (UUID PK), upload_ids (UUID[]), caption, hashtags (TEXT[]), platforms (TEXT[]: instagram/linkedin/facebook/tiktok), status (draft/pending_approval/approved/rejected/published/failed), platform_crops (JSONB: { instagram: {url, aspect}, linkedin: {url, aspect}, tiktok: {url, aspect} }), ai_score (REAL), created_at, updated_at
   - `published_posts` table: id (UUID PK), draft_id (FK), platform, platform_post_id, published_at, engagement_data (JSONB nullable)
   - Indexes: drafts(status), drafts(created_at), published_posts(platform, published_at)
3. Implement Elysia routes:
   - `POST /api/v1/social/upload` — multipart upload, store originals in R2 `social/` prefix, create upload records, trigger AI curation pipeline
   - `GET /api/v1/social/drafts` — list drafts with status filter, paginated
   - `GET /api/v1/social/drafts/:id` — draft detail with image URLs and caption
   - `POST /api/v1/social/drafts/:id/approve` — transition status to approved
   - `POST /api/v1/social/drafts/:id/reject` — transition to rejected with optional reason
   - `POST /api/v1/social/drafts/:id/publish` — publish to selected platforms
   - `GET /api/v1/social/published` — list published posts with engagement data
   - `GET /metrics` — Prometheus metrics (prom-client)
   - `GET /health/live`, `GET /health/ready`
4. Effect Service layer:
   - `ImageCurationService` (Effect.Service): score uploaded images using OpenAI Vision API, select top 5-10 by composition quality, lighting, subject clarity
   - `CaptionService` (Effect.Service): generate platform-specific captions using OpenAI/Claude — event context, equipment featured, relevant hashtags
   - `CropService` (Effect.Service): generate platform-specific crops — Instagram (1:1 + 9:16 Story), LinkedIn (1.91:1), TikTok (9:16) — using sharp library
   - `InstagramService` (Effect.Service): publish via Instagram Graph API
   - `LinkedInService` (Effect.Service): publish via LinkedIn API
   - `FacebookService` (Effect.Service): publish via Facebook Graph API
   - `TikTokService` (Effect.Service): publish via TikTok API
   - All publish services use `Effect.retry` with exponential backoff (base 1s, max 30s, 3 attempts)
5. AI Curation Pipeline (triggered on upload):
   - Receive batch of photos → ImageCurationService scores each (0-100) → select top images → CropService generates platform crops → CaptionService generates captions → create Draft with status 'pending_approval'
6. Publishing Pipeline:
   - On approve → publish to each selected platform using respective service
   - Record platform_post_id on success
   - On any platform failure, record partial success (some platforms published, others failed)
7. Effect Schema validation on all request/response types.
8. R2 integration via @aws-sdk/client-s3 (S3-compatible).
9. RBAC middleware: validate JWT, check role from sigma1-rbac-roles ConfigMap.
10. Kubernetes Deployment: namespace `sigma1`, 1 replica, envFrom sigma1-infra-endpoints.

### Subtasks
- [ ] Initialize Bun/Elysia/Effect project with TypeScript configuration: Scaffold the Social Media Engine project with Bun runtime, Elysia web framework, Effect 3.x, TypeScript 5.x strict mode, and biome.js for linting/formatting. Set up the project structure with standard directories for routes, services, migrations, and tests.
- [ ] Create database migrations for social schema (uploads, drafts, published_posts): Implement database migrations using drizzle-orm (or kysely) for the `social` schema, defining the uploads, drafts, and published_posts tables with all columns, types, indexes, and foreign key constraints as specified.
- [ ] Implement R2 storage integration via @aws-sdk/client-s3: Create an R2StorageService as an Effect service that handles uploading files to Cloudflare R2 (S3-compatible) under the `social/` prefix, generating presigned URLs for retrieval, and deleting objects.
- [ ] Implement upload endpoint and draft listing/detail endpoints: Build the Elysia routes for multipart image upload (POST /api/v1/social/upload), draft listing with pagination and status filter (GET /api/v1/social/drafts), and draft detail (GET /api/v1/social/drafts/:id). Integrate R2 for file storage and database for record persistence.
- [ ] Implement ImageCurationService with OpenAI Vision API scoring: Create the ImageCurationService as an Effect service that scores uploaded images using OpenAI Vision API on composition quality, lighting, and subject clarity, returning scores 0-100 and selecting the top images from a batch.
- [ ] Implement CropService with sharp for platform-specific image crops: Create the CropService as an Effect service that generates platform-specific image crops using the sharp library: Instagram square (1:1), Instagram Story (9:16), LinkedIn (1.91:1), and TikTok (9:16).
- [ ] Implement CaptionService with AI-powered caption generation: Create the CaptionService as an Effect service that generates platform-specific captions with relevant hashtags using OpenAI/Claude, incorporating event context and equipment details.
- [ ] Implement AI curation pipeline orchestrating scoring, cropping, and captioning: Build the end-to-end AI curation pipeline that is triggered after image upload: score images via ImageCurationService, select top images, generate platform crops via CropService, generate captions via CaptionService, and create a Draft record with status 'pending_approval'.
- [ ] Implement Instagram publishing service: Create the InstagramService as an Effect service that publishes images to Instagram via the Instagram Graph API, handling the container creation and publish flow with Effect.retry and exponential backoff.
- [ ] Implement LinkedIn publishing service: Create the LinkedInService as an Effect service that publishes images to LinkedIn via the LinkedIn API with Effect.retry and exponential backoff.
- [ ] Implement Facebook publishing service: Create the FacebookService as an Effect service that publishes images to Facebook Pages via the Facebook Graph API with Effect.retry and exponential backoff.
- [ ] Implement TikTok publishing service: Create the TikTokService as an Effect service that publishes content to TikTok via the TikTok API with Effect.retry and exponential backoff.
- [ ] Implement approval workflow endpoints and multi-platform publishing pipeline: Build the approve, reject, and publish endpoints with draft status state machine transitions, and the publishing pipeline that dispatches to multiple platform services concurrently with partial failure handling.
- [ ] Implement RBAC middleware with JWT validation: Create Elysia middleware that validates JWT tokens from incoming requests and checks user roles against the sigma1-rbac-roles ConfigMap for authorization on social media endpoints.
- [ ] Implement Prometheus metrics and health endpoints: Add Prometheus metrics collection (prom-client) and Kubernetes health/readiness endpoints to the Elysia application.
- [ ] Define Effect Schema validators for all request/response types: Create comprehensive Effect Schema definitions for all API request bodies, query parameters, and response types to ensure type-safe validation across all endpoints.
- [ ] Create Kubernetes deployment manifest for social-engine: Write the Kubernetes Deployment, Service, and related manifests for the social-engine in the sigma1 namespace with proper resource configuration, environment injection from sigma1-infra-endpoints ConfigMap, and health probes.