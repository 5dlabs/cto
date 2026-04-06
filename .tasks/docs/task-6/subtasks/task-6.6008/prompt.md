Implement subtask 6008: Wire up publish endpoint, published posts listing, metrics, and schema validation

## Objective
Implement the publish and published-posts endpoints that orchestrate multi-platform publishing, record results, and add Prometheus metrics and health probes.

## Steps
1. Implement POST `/api/v1/social/drafts/:id/publish` handler:
   - Validate draft is in 'approved' status.
   - Publish to all target platforms concurrently using the platform-specific clients (Instagram, LinkedIn, Facebook).
   - Record each platform's publish result in `published_posts` table.
   - Update draft status to 'published'.
   - Handle partial failures (some platforms succeed, some fail) — store successes, report failures in response.
2. Implement GET `/api/v1/social/published` handler:
   - List all published posts with pagination, filterable by platform.
   - Join with drafts table to include original caption and images.
3. Add Prometheus metrics:
   - `social_requests_total` (counter, labels: endpoint, status)
   - `social_publish_total` (counter, labels: platform, success/failure)
   - `social_ai_curation_duration_seconds` (histogram)
   - Expose GET `/metrics` endpoint.
4. Validate all endpoint request/response schemas with Effect.Schema.
5. Ensure all error responses follow consistent JSON structure.

## Validation
Integration tests: publish an approved draft posts to all target platforms and records results in published_posts; partial failure (one platform fails) still publishes to others and returns partial success; GET /published returns paginated list with platform details; GET /metrics returns valid Prometheus format; publishing a non-approved draft returns 409.