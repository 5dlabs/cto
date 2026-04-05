Implement subtask 6009: Implement Instagram publishing service

## Objective
Create the InstagramService as an Effect service that publishes images to Instagram via the Instagram Graph API, handling the container creation and publish flow with Effect.retry and exponential backoff.

## Steps
1. Create `src/services/publishing/InstagramService.ts` as an Effect.Service.
2. Instagram Graph API publishing flow:
   a. Create a media container: `POST /{ig-user-id}/media` with `image_url` (presigned R2 URL), `caption`, and `access_token`.
   b. Poll container status until ready (or timeout after 60s).
   c. Publish the container: `POST /{ig-user-id}/media_publish` with `creation_id` and `access_token`.
3. Define service interface:
   - `publish(input: { imageUrl: string, caption: string, accessToken: string, userId: string }): Effect.Effect<{ platformPostId: string }, InstagramPublishError>`
4. Wrap the entire flow with `Effect.retry` using `Schedule.exponential('1 seconds').pipe(Schedule.intersect(Schedule.recurs(3)), Schedule.union(Schedule.spaced('30 seconds')))`.
5. Define `InstagramPublishError` as a tagged Effect error with status code and API error details.
6. Create `InstagramServiceLive` layer reading credentials from environment.
7. Handle rate limiting (HTTP 429) by respecting Retry-After header.

## Validation
Unit test with mocked Graph API: verify publish flow creates container, polls status, and publishes. Verify returned platformPostId matches Graph API response. Retry test: mock API to fail twice with 500, then succeed — verify Effect.retry produces success. Rate limit test: mock 429 response, verify retry respects delay.