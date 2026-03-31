Implement subtask 6010: Implement Facebook publishing integration

## Objective
Create an Effect.Service for the Facebook Graph API integration to publish approved posts to a Facebook page.

## Steps
1. Create `src/services/FacebookService.ts` as an Effect.Service tag with methods: publishPhoto(photoUrl: string, caption: string, pageAccessToken: string) → Effect<PublishResult, PlatformPublishError>, getPostInsights(postId: string, pageAccessToken: string) → Effect<Engagement, PlatformPublishError>. 2. Facebook Graph API flow: POST to /{page-id}/photos with url (photo URL) and message (caption) fields. 3. Use page access tokens (not user tokens) for publishing. 4. Handle Facebook-specific errors: rate limiting (#4, #17), permission errors (#200), expired tokens. 5. Facebook caption has no hard limit but keep under 63206 chars. 6. Implement getPostInsights using /{post-id}/insights for reach, impressions, engagement. 7. Wrap all HTTP calls in Effect.tryPromise with FacebookPublishError, FacebookAuthError tags.

## Validation
Mock Facebook Graph API and verify single-step photo publish flow. Verify page access token is used (not user token). Verify rate limit error handling with proper backoff. Verify insights retrieval returns engagement metrics.