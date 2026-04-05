Implement subtask 6009: Implement Effect.Services for Facebook publishing

## Objective
Build an Effect.Service for publishing content to Facebook via the Facebook Graph API, including photo upload, post creation on a page, and status tracking.

## Steps
1. Create src/integrations/facebook.ts.
2. Define Effect.Service `FacebookPublisher` with methods: publish(content: PublishableContent) -> Effect<PublishResult>, getPostStatus(postId: string) -> Effect<PostStatus>.
3. Implement Facebook Graph API: upload photos to page -> create post with photo IDs.
4. Support multi-photo posts.
5. Post as page (using page access token, not user token).
6. Handle Facebook page token management and refresh.
7. Handle API errors, rate limits, and content moderation rejections.
8. Implement mock for testing.

## Validation
Unit tests with mocked Facebook API verify correct photo upload and post creation flow; multi-photo posts aggregate correctly; page token is used instead of user token; error handling covers common Facebook API error codes.