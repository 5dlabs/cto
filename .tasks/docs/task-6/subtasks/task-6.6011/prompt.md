Implement subtask 6011: Implement API endpoints for upload, drafts, and publishing lifecycle

## Objective
Implement all Elysia HTTP endpoints for the social media content lifecycle: upload photos, list/view/approve/reject/publish drafts, and list published content. Wire up all services and pipelines.

## Steps
1. Create src/handlers/social.ts.
2. POST /api/v1/social/upload: Accept multipart file uploads, store in S3/R2 via StorageService, trigger AI curation pipeline, create a draft with AI-generated caption. Return 201 with draft.
3. GET /api/v1/social/drafts: List all drafts with pagination, filterable by status. Return 200.
4. GET /api/v1/social/drafts/:id: Get a single draft with full details including AI metadata. Return 200 or 404.
5. POST /api/v1/social/drafts/:id/approve: Trigger approval workflow, update status. Return 200.
6. POST /api/v1/social/drafts/:id/reject: Accept rejection reason, update status. Return 200.
7. POST /api/v1/social/drafts/:id/publish: Publish approved draft to all target platforms concurrently using Effect.allSettled. Create published records. Return 200 with per-platform results.
8. GET /api/v1/social/published: List published content with pagination, filterable by platform. Return 200.
9. Add request validation using @effect/schema.
10. Add error handling middleware that maps Effect failures to HTTP status codes.
11. Wire all routes into the main Elysia app with dependency injection of Effect services.

## Validation
Integration tests verify: upload creates draft with AI caption; drafts list returns correct data with pagination; approve/reject update status; publish sends to all target platforms and creates published records; published list shows results; validation rejects invalid requests; 404 for unknown drafts.