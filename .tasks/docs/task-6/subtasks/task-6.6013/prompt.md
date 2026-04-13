<identity>
You are nova working on subtask 6013 of task 6.
</identity>

<context>
<scope>
Implement the publish endpoint that runs all four platform Effect services in parallel for an approved draft, records per-platform social_posts rows, and exposes GET /api/v1/social/published for portfolio sync.
</scope>
</context>

<implementation_plan>
POST /api/v1/social/drafts/:id/publish handler: verify draft status='approved', else return 409. Run Effect.all([InstagramService.publishPost(draft), LinkedInService.publishPost(draft), TikTokService.uploadVideo(draft), FacebookService.publishPost(draft)], {concurrency:'unbounded'}). Use Effect.either on each to capture per-platform success/failure without aborting others. For each result: on Right insert social_posts with status='published'; on Left insert with status='failed' and error_text. Update social_drafts status='published'. Return 200 {platforms: {instagram:'published', linkedin:'published', tiktok:'failed', facebook:'published'}}. GET /api/v1/social/published: JOIN social_posts and social_drafts, return posts with status='published', ordered by published_at DESC.
</implementation_plan>

<validation>
POST publish on approved draft → 200. GET /api/v1/social/published includes the new post. Test partial failure: mock LinkedIn to fail permanently; verify response shows linkedin:'failed', other platforms show 'published', and social_posts has 4 rows (3 published, 1 failed). Verify draft status='published' even with one platform failure.
</validation>