Implement subtask 6008: Implement Effect Service clients for Instagram, LinkedIn, Facebook, and TikTok APIs

## Objective
Build four separate Effect.Service implementations for publishing content to each social media platform via their respective APIs, all with exponential backoff retry logic.

## Steps
1. Create `src/services/platforms/InstagramService.ts`:
   - Effect.Service tag: InstagramService.
   - Method `publishPost(imageUrls: string[], caption: string, hashtags: string[])`: Effect<PublishResult, PlatformPublishError>
     - Use Instagram Graph API: POST /{ig-user-id}/media (for container), then POST /{ig-user-id}/media_publish.
     - For carousel (multiple images): create child containers, then publish carousel container.
     - Return { postId: string, postUrl: string }.
   - Wrap in Effect.retry(Schedule.exponential('1 second').pipe(Schedule.intersect(Schedule.recurs(3)))).
2. Create `src/services/platforms/LinkedInService.ts`:
   - Method `publishPost(imageUrls: string[], caption: string)`: Effect<PublishResult, PlatformPublishError>
     - Use LinkedIn API: Register image upload, upload image, create ugcPost with image.
     - Return { postId, postUrl }.
3. Create `src/services/platforms/FacebookService.ts`:
   - Method `publishPost(pageId: string, imageUrls: string[], caption: string)`: Effect<PublishResult, PlatformPublishError>
     - Use Facebook Graph API: POST /{page-id}/photos for single, or unpublished photos + POST /{page-id}/feed for multi-photo.
     - Return { postId, postUrl }.
4. Create `src/services/platforms/TikTokService.ts`:
   - Method `publishVideo(videoUrl: string, caption: string, hashtags: string[])`: Effect<PublishResult, PlatformPublishError>
     - Use TikTok Content Posting API: init upload, upload video, publish.
     - For photo mode (if supported): use photo post endpoint.
     - Return { postId, postUrl }.
5. Each service:
   - Takes API credentials (access tokens, app IDs) from environment.
   - Uses `Effect.tryPromise` for all HTTP calls (via fetch or undici).
   - Tagged error types: InstagramPublishError, LinkedInPublishError, etc.
   - All have a `.live` layer file.
6. Create `src/services/platforms/PlatformRouter.ts`:
   - Method `publishToPlatform(platform: Platform, draft: Draft)`: Effect<PublishResult, PlatformPublishError>
   - Routes to the correct service based on platform field.

## Validation
Unit test each service with mocked HTTP responses: (1) InstagramService: mock Graph API container creation and publish, verify correct API calls sequence for single and carousel posts. (2) LinkedInService: mock image registration and ugcPost creation. (3) FacebookService: mock multi-photo publish flow. (4) TikTokService: mock upload init, upload, and publish sequence. (5) Test retry logic: mock 500 response twice then 200, verify 3 attempts made with exponential delay. (6) Test PlatformRouter routes to correct service for each platform string.