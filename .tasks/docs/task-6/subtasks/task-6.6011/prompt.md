Implement subtask 6011: Implement publish endpoint with multi-platform orchestration

## Objective
Build the POST /api/v1/social/drafts/:id/publish endpoint that orchestrates publishing an approved draft to all target platforms and the GET /api/v1/social/published endpoint.

## Steps
1. In `src/routes/drafts.ts`, implement `POST /api/v1/social/drafts/:id/publish`: a) Validate draft exists and status is APPROVED. b) For each target platform in the draft, invoke the corresponding Publisher Effect.Service. c) Use Effect.all with concurrency to publish to platforms in parallel. d) For each successful publish, create a PublishedPost record in PostgreSQL. e) Update draft status to PUBLISHED. f) Return 200 with array of PublishResults (per-platform success/failure). g) Handle partial failures: if Instagram succeeds but TikTok fails, still record the Instagram success and report TikTok failure. 2. Implement `GET /api/v1/social/published` — lists all published posts with pagination, filterable by platform. 3. Validate all with Effect.Schema.

## Validation
POST /publish on APPROVED draft publishes to all target platforms; PublishedPost records are created in DB; draft status changes to PUBLISHED; partial platform failures are handled (successful platforms are recorded); GET /published returns paginated list; publishing a non-APPROVED draft returns 409.