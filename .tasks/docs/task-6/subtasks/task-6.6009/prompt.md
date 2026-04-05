Implement subtask 6009: Implement publish endpoint and portfolio sync

## Objective
Build the POST /api/v1/social/drafts/:id/publish endpoint that orchestrates multi-platform publishing and the GET /api/v1/social/published endpoint, plus website portfolio synchronization.

## Steps
1. POST /api/v1/social/drafts/:id/publish: Validate draft is in 'approved' status. Read platform_targets from the draft (array of 'instagram', 'linkedin', 'facebook'). 2. Invoke each platform's PublishingProvider concurrently using Effect.all with concurrency control. 3. For each successful publish, insert a record into published_posts table with platform, platform_post_id, url, published_at. 4. Update draft status to 'published' only if at least one platform succeeds. Store partial failure details in the draft's metadata. 5. GET /api/v1/social/published: Return paginated list of published posts with platform details, optionally filtered by platform or date range. 6. Implement portfolio sync: after successful publish, call an internal webhook or write to a sync queue that the website can consume to update the portfolio gallery. For v1, write a JSON manifest to S3 at a known key (e.g., portfolio/manifest.json) containing all published posts with image URLs and captions. 7. Return detailed publish results including per-platform success/failure status.

## Validation
Integration test: publish an approved draft to mocked platforms, verify published_posts records are created, draft status transitions to 'published'. Test partial failure: one platform fails, others succeed — verify draft is still marked published with failure details. GET /published returns correct records. Verify portfolio manifest is written to S3.