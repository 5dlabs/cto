Implement subtask 6012: Implement TikTok publishing service

## Objective
Create the TikTokService as an Effect service that publishes content to TikTok via the TikTok API with Effect.retry and exponential backoff.

## Steps
1. Create `src/services/publishing/TikTokService.ts` as an Effect.Service.
2. TikTok Content Posting API flow:
   a. Initialize photo upload: `POST /v2/post/publish/content/init/` with `post_info` (title/description, privacy_level) and `source_info` (photo_images with URLs).
   b. Check publish status: `POST /v2/post/publish/status/fetch/` with `publish_id`.
3. Define service interface:
   - `publish(input: { imageUrls: string[], caption: string, accessToken: string }): Effect.Effect<{ platformPostId: string }, TikTokPublishError>`
4. Wrap with `Effect.retry` — exponential backoff, base 1s, max 30s, 3 attempts.
5. Define `TikTokPublishError` as tagged Effect error.
6. Create `TikTokServiceLive` layer reading credentials from environment.
7. Handle TikTok-specific requirements: photo posts may require multiple images (carousel), respect content policies.

## Validation
Unit test with mocked TikTok API: verify init and status check flow. Verify platformPostId from response. Retry test: mock failures then success, verify retry logic works.