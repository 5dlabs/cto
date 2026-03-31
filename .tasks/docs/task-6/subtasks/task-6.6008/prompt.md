Implement subtask 6008: Implement Instagram publishing integration

## Objective
Create an Effect.Service for the Instagram Graph API integration to publish approved social media posts to Instagram.

## Steps
1. Create `src/services/InstagramService.ts` as an Effect.Service tag with methods: publishPhoto(photoUrl: string, caption: string, accessToken: string) → Effect<PublishResult, PlatformPublishError>, getPostInsights(postId: string, accessToken: string) → Effect<Engagement, PlatformPublishError>. 2. Instagram Graph API flow: POST to /{ig-user-id}/media with image_url and caption to create a media container → POST to /{ig-user-id}/media_publish with creation_id to publish. 3. Implement token refresh logic for long-lived tokens. 4. PublishResult schema: { platformPostId: string, permalink: string, publishedAt: Date }. 5. Handle Instagram-specific errors: rate limits (error code 4), media format errors, caption length limits (2200 chars). 6. Wrap all HTTP calls in Effect.tryPromise with proper error tagging.

## Validation
Mock Instagram Graph API and verify the two-step publish flow (create container then publish). Verify token refresh is triggered when token is near expiry. Verify caption exceeding 2200 chars returns a validation error. Verify rate limit errors are properly caught and tagged.