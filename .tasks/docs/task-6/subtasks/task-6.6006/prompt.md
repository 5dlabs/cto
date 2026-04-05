Implement subtask 6006: Implement Effect.Services for Instagram publishing

## Objective
Build an Effect.Service for publishing content to Instagram via the Instagram Graph API, including image upload, caption posting, and status tracking.

## Steps
1. Create src/integrations/instagram.ts.
2. Define Effect.Service `InstagramPublisher` with methods: publish(content: PublishableContent) -> Effect<PublishResult>, getPostStatus(postId: string) -> Effect<PostStatus>.
3. PublishableContent: image_urls (string[]), caption, hashtags, scheduled_at (optional).
4. PublishResult: platform_post_id, published_url, status.
5. Implement Instagram Graph API flow: create media container -> publish media container.
6. Handle carousel posts (multiple images) via the carousel container API.
7. Manage Instagram access token refresh.
8. Handle API errors: rate limits, invalid media, permission errors.
9. Implement mock for testing.

## Validation
Unit tests with mocked Instagram API verify correct API call sequence (container creation then publish); carousel posts send correct media array; token refresh is triggered on 401; errors map to typed Effect failures.