Implement subtask 6008: Implement Effect.Services for TikTok publishing

## Objective
Build an Effect.Service for publishing content to TikTok via the TikTok Content Posting API, including video/image upload and status tracking.

## Steps
1. Create src/integrations/tiktok.ts.
2. Define Effect.Service `TikTokPublisher` with methods: publish(content: PublishableContent) -> Effect<PublishResult>, getPostStatus(postId: string) -> Effect<PostStatus>.
3. Implement TikTok Content Posting API: initialize upload -> upload content -> publish.
4. Support photo posts (TikTok photo mode) since the primary content is event photos.
5. Handle TikTok OAuth2 token management.
6. Map captions to TikTok's format (character limits, hashtag style).
7. Handle API errors, content policy rejections, and rate limits.
8. Implement mock for testing.

## Validation
Unit tests with mocked TikTok API verify correct upload flow; photo mode posts are correctly formatted; OAuth token refresh works; content policy rejection errors are properly surfaced.