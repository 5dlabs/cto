## Develop Social Media Engine (Nova - Node.js/Elysia + Effect)

### Objective
Create the Social Media Engine for AI-powered photo curation, caption generation, approval workflow, and multi-platform publishing. Integrates with Instagram, LinkedIn, Facebook, and OpenAI/Claude.

### Ownership
- Agent: nova
- Stack: Node.js 20+, Elysia 1.x, Effect TS
- Priority: medium
- Status: pending
- Dependencies: 1

### Implementation Details
{"steps": ["Initialize Elysia project with Effect 3.x and TypeScript 5.x.", "Connect to PostgreSQL and S3/R2 using ConfigMap endpoints.", "Implement endpoints: /api/v1/social/upload, /api/v1/social/drafts, /api/v1/social/drafts/:id, /api/v1/social/drafts/:id/approve, /api/v1/social/drafts/:id/reject, /api/v1/social/drafts/:id/publish, /api/v1/social/published.", "Integrate Effect.Service for Instagram, LinkedIn, TikTok, Facebook APIs.", "Implement AI curation pipeline using OpenAI/Claude for captioning.", "Add approval workflow with Signal notification to Morgan.", "Publish approved content to platforms and sync to portfolio.", "Add Effect.Schema validation for all requests/responses."]}

### Subtasks
- [ ] Scaffold Elysia project with Effect 3.x and TypeScript 5.x: Initialize the Social Media Engine project with Elysia 1.x, Effect 3.x, and TypeScript 5.x. Set up the project structure, tsconfig, package.json with Bun as runtime, and Effect layer/service architecture. Include health check endpoint at /api/v1/social/health and Prometheus metrics endpoint.
- [ ] Implement PostgreSQL connection layer with Effect: Create an Effect Layer for PostgreSQL connectivity using the connection string from the ConfigMap. Implement the database client service, connection pool, and run the social media schema migrations.
- [ ] Implement S3/R2 object storage layer with Effect: Create an Effect Layer for S3-compatible object storage (R2 or S3) for photo uploads, thumbnail generation, and retrieval. Read bucket configuration from ConfigMap endpoints.
- [ ] Implement photo upload endpoint and storage pipeline: Build the POST /api/v1/social/upload endpoint that accepts photo files, stores originals in S3/R2, generates thumbnails, and persists metadata to PostgreSQL.
- [ ] Implement AI curation service with OpenAI/Claude for caption generation: Create an Effect.Service abstraction for AI-powered caption generation. Integrate with OpenAI and/or Claude APIs to analyze uploaded photos and generate platform-specific captions.
- [ ] Implement draft management endpoints and AI-powered draft creation: Build the draft CRUD endpoints that tie together photo uploads and AI caption generation to create social media drafts ready for approval.
- [ ] Implement approval workflow with Signal notification: Build the approval/rejection endpoints and integrate with Signal messaging to notify Morgan when drafts are ready for review, allowing approve/reject actions.
- [ ] Implement Instagram publishing integration: Create an Effect.Service for the Instagram Graph API integration to publish approved social media posts to Instagram.
- [ ] Implement LinkedIn publishing integration: Create an Effect.Service for the LinkedIn API integration to publish approved posts to a LinkedIn profile or company page.
- [ ] Implement Facebook publishing integration: Create an Effect.Service for the Facebook Graph API integration to publish approved posts to a Facebook page.
- [ ] Implement publish orchestration endpoint and portfolio sync: Build the POST /api/v1/social/drafts/:id/publish endpoint that orchestrates publishing approved drafts to their target platform, records results, and syncs published content to the portfolio.
- [ ] Add Effect.Schema validation for all request/response contracts: Define and enforce Effect.Schema validators on every Elysia route for request bodies, query parameters, path parameters, and response shapes across all social media endpoints.
- [ ] Write integration tests for the complete social media workflow: Create end-to-end integration tests covering the full workflow: photo upload → AI curation → draft creation → approval → publish → portfolio sync.