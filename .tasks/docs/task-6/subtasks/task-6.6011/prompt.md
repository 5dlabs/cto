Implement subtask 6011: Implement Facebook publishing service

## Objective
Create the FacebookService as an Effect service that publishes images to Facebook Pages via the Facebook Graph API with Effect.retry and exponential backoff.

## Steps
1. Create `src/services/publishing/FacebookService.ts` as an Effect.Service.
2. Facebook Graph API publishing flow:
   a. Upload photo: `POST /{page-id}/photos` with `url` (presigned R2 URL), `caption` (message), and `access_token` (Page access token).
   b. Alternatively for more control: upload as unpublished, then publish.
3. Define service interface:
   - `publish(input: { imageUrl: string, caption: string, accessToken: string, pageId: string }): Effect.Effect<{ platformPostId: string }, FacebookPublishError>`
4. Wrap with `Effect.retry` — exponential backoff, base 1s, max 30s, 3 attempts.
5. Define `FacebookPublishError` as tagged Effect error.
6. Create `FacebookServiceLive` layer reading credentials from environment.

## Validation
Unit test with mocked Graph API: verify photo upload call with correct parameters. Verify platformPostId from response. Retry test: mock failures then success, verify retry logic works.