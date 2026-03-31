Implement subtask 6011: Implement publish orchestration endpoint and portfolio sync

## Objective
Build the POST /api/v1/social/drafts/:id/publish endpoint that orchestrates publishing approved drafts to their target platform, records results, and syncs published content to the portfolio.

## Steps
1. Create `src/routes/publish.ts` with POST /api/v1/social/drafts/:id/publish. 2. Validate draft status is 'approved' (return 409 if not). 3. Load platform credentials from `social_platform_credentials` table (decrypt tokens using a shared encryption key from env). 4. Route to the correct platform service based on draft.platform: instagram → InstagramService.publishPhoto, linkedin → LinkedInService.publishPost, facebook → FacebookService.publishPhoto. 5. On success: insert record into `social_published` table with platform_post_id, permalink, published_at → update draft status to 'published'. 6. On failure: keep draft as 'approved', return error details so it can be retried. 7. Implement GET /api/v1/social/published endpoint listing all published posts with filtering by platform and date range. 8. Portfolio sync: after successful publish, send an HTTP POST to the portfolio service (or emit an event if event-driven is chosen) with the published photo URL, caption, platform, and permalink so it appears on the website. 9. Add Prometheus counters: social_posts_published_total (by platform), social_publish_errors_total (by platform).

## Validation
Publish an approved Instagram draft and verify social_published record is created with platform_post_id. Attempt to publish a non-approved draft and verify 409. Verify portfolio sync HTTP call is made. GET /published returns the published post. Verify publish failure does not transition draft to published.