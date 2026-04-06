## Develop Social Media Engine (Nova - Node.js/Elysia + Effect)

### Objective
Create the Social Media Engine for AI curation, caption generation, approval workflow, and multi-platform publishing.

### Ownership
- Agent: nova
- Stack: Node.js 20+/Elysia + Effect
- Priority: medium
- Status: pending
- Dependencies: 1

### Implementation Details
{"steps": ["Initialize Node.js Elysia project with Effect 3.x, connect to PostgreSQL and S3/R2 via 'sigma1-infra-endpoints'", "Implement endpoints: /api/v1/social/upload, /api/v1/social/drafts, /api/v1/social/drafts/:id, /api/v1/social/drafts/:id/approve, /api/v1/social/drafts/:id/reject, /api/v1/social/drafts/:id/publish, /api/v1/social/published", "Integrate OpenAI/Claude for caption generation and image curation", "Implement Effect.Service for Instagram, LinkedIn, TikTok, Facebook publishing", "Add approval workflow: send draft to Morgan for Signal approval", "Ensure platform-specific cropping and portfolio sync to website", "Validate all requests/responses with Effect.Schema"]}

### Subtasks
- [ ] Initialize Elysia project with Effect 3.x, PostgreSQL, and S3/R2 connectivity: Scaffold the Node.js Elysia project with Effect 3.x integration, establish PostgreSQL connection pool and S3/R2 object storage client, reading all configuration from the 'sigma1-infra-endpoints' ConfigMap.
- [ ] Define Effect.Schema models and database migrations for social media entities: Create Effect.Schema definitions for all domain entities (Upload, Draft, PublishedPost) and write PostgreSQL migrations for the corresponding tables.
- [ ] Implement image upload endpoint with S3/R2 storage: Build the POST /api/v1/social/upload endpoint that accepts image files, stores them in S3/R2, and creates upload records in PostgreSQL.
- [ ] Integrate AI service for caption generation and image curation: Build an Effect.Service that interfaces with OpenAI or Claude to generate social media captions and curate/analyze uploaded images for suitability.
- [ ] Implement draft management CRUD endpoints: Build the draft management endpoints: GET /api/v1/social/drafts, GET /api/v1/social/drafts/:id, and draft creation flow that combines uploads with AI-generated captions into reviewable drafts.
- [ ] Implement approval workflow with Signal notification to Morgan: Build the approve/reject endpoints and integrate Signal messaging to notify Morgan when drafts need review and when approval decisions are made.
- [ ] Implement Instagram publishing Effect.Service: Build the Effect.Service implementation for publishing posts to Instagram, including platform-specific image cropping and the Instagram Graph API integration.
- [ ] Implement LinkedIn publishing Effect.Service: Build the Effect.Service implementation for publishing posts to LinkedIn, including the LinkedIn Marketing API integration for company page posts.
- [ ] Implement TikTok publishing Effect.Service: Build the Effect.Service implementation for publishing content to TikTok, including the TikTok Content Posting API integration.
- [ ] Implement Facebook publishing Effect.Service: Build the Effect.Service implementation for publishing posts to Facebook, including the Facebook Graph API integration for page posts.
- [ ] Implement publish endpoint with multi-platform orchestration: Build the POST /api/v1/social/drafts/:id/publish endpoint that orchestrates publishing an approved draft to all target platforms and the GET /api/v1/social/published endpoint.
- [ ] Implement portfolio sync to website: Build the service that syncs published social media posts to the company website portfolio, ensuring published content is reflected on the website gallery.
- [ ] Write end-to-end integration tests for full social media workflow: Create comprehensive integration tests covering the complete flow: upload → AI curation → draft creation → approval → publish → portfolio sync, with mocked external services.