Implement subtask 6013: Implement POST /api/v1/social/drafts/:id/publish orchestration and GET /api/v1/social/published

## Objective
Implement the publish endpoint that runs all four platform Effect services in parallel for an approved draft, records per-platform social_posts rows, and exposes GET /api/v1/social/published for portfolio sync.

## Steps
POST /api/v1/social/drafts/:id/publish handler: verify draft status='approved', else return 409. Run Effect.all([InstagramService.publishPost(draft), LinkedInService.publishPost(draft), TikTokService.uploadVideo(draft), FacebookService.publishPost(draft)], {concurrency:'unbounded'}). Use Effect.either on each to capture per-platform success/failure without aborting others. For each result: on Right insert social_posts with status='published'; on Left insert with status='failed' and error_text. Update social_drafts status='published'. Return 200 {platforms: {instagram:'published', linkedin:'published', tiktok:'failed', facebook:'published'}}. GET /api/v1/social/published: JOIN social_posts and social_drafts, return posts with status='published', ordered by published_at DESC.

## Validation
POST publish on approved draft → 200. GET /api/v1/social/published includes the new post. Test partial failure: mock LinkedIn to fail permanently; verify response shows linkedin:'failed', other platforms show 'published', and social_posts has 4 rows (3 published, 1 failed). Verify draft status='published' even with one platform failure.