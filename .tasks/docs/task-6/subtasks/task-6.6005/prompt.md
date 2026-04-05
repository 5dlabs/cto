Implement subtask 6005: Implement multi-platform publishing services using Effect.Service

## Objective
Build Effect.Service implementations for publishing content to Instagram, LinkedIn, TikTok, and Facebook, each as a separate service with platform-specific API integration.

## Steps
1. Create `src/services/publishing/` directory with separate files per platform.
2. Define a common `PublishingService` Effect.Service interface: `publish(draft: Draft) -> Effect.Effect<PublishResult, PublishError>`, `getPostStatus(externalId: string) -> Effect.Effect<PostStatus, PublishError>`.
3. Implement `InstagramPublishService`:
   - Use Instagram Graph API / Basic Display API.
   - Handle media upload (image container creation → publish), caption posting.
   - Store external_post_id on success.
4. Implement `LinkedInPublishService`:
   - Use LinkedIn Marketing API for company page posts.
   - Handle image upload and ugcPost creation.
5. Implement `TikTokPublishService`:
   - Use TikTok Content Posting API.
   - Handle video/image upload flow.
6. Implement `FacebookPublishService`:
   - Use Facebook Graph API for page posts.
   - Handle photo and text post creation.
7. Each implementation: authenticate with platform-specific OAuth tokens from secrets, handle rate limits, return structured PublishResult { externalPostId, platform, publishedUrl, publishedAt }.
8. Implement POST `/api/v1/social/publish` endpoint:
   - Accept draft_id and target platform(s).
   - Call the appropriate publishing service(s).
   - Update `published_posts` table.
   - Return publish results.
9. Implement GET `/api/v1/social/posts` — list published posts with platform filter.

## Validation
Mock each platform API. Verify publishing a draft to each platform creates the correct API calls and persists results. Verify multi-platform publish hits all selected platforms. Test error handling when one platform fails but others succeed. Verify GET /posts returns published content.