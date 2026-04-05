Implement subtask 6010: Implement LinkedIn publishing service

## Objective
Create the LinkedInService as an Effect service that publishes images to LinkedIn via the LinkedIn API with Effect.retry and exponential backoff.

## Steps
1. Create `src/services/publishing/LinkedInService.ts` as an Effect.Service.
2. LinkedIn API publishing flow:
   a. Register an image upload: `POST /v2/assets?action=registerUpload` with owner URN.
   b. Upload the image binary to the upload URL returned.
   c. Create a post: `POST /v2/ugcPosts` (or /v2/posts for new API) with the asset URN, caption text, and visibility settings.
3. Define service interface:
   - `publish(input: { imageBuffer: Buffer, caption: string, accessToken: string, organizationUrn: string }): Effect.Effect<{ platformPostId: string }, LinkedInPublishError>`
4. Wrap with `Effect.retry` — exponential backoff, base 1s, max 30s, 3 attempts.
5. Define `LinkedInPublishError` as tagged Effect error.
6. Create `LinkedInServiceLive` layer reading credentials from environment.

## Validation
Unit test with mocked LinkedIn API: verify the 3-step flow (register upload, upload binary, create post) executes correctly. Verify platformPostId is extracted from response. Retry test: mock 500 error twice then success — verify retry completes successfully.