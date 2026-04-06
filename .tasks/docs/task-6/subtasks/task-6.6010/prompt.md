Implement subtask 6010: Implement Facebook publishing Effect.Service

## Objective
Build the Effect.Service implementation for publishing posts to Facebook, including the Facebook Graph API integration for page posts.

## Steps
1. In `src/services/publishers/facebook.ts`, create an Effect.Service `FacebookPublisher` implementing the `Publisher` interface. 2. Implement `publish(draft: Draft, images: Upload[]) -> Effect<PublishResult>`: a) Upload photos to Facebook page via Graph API /{page-id}/photos. b) Create a post with uploaded photos and caption via /{page-id}/feed. c) For multiple images, create an album post or multi-photo post. d) Return PublishResult with platform_post_id and post_url. 3. Handle Facebook page access token management. 4. Handle Facebook API rate limits and error responses. 5. Format captions appropriately for Facebook.

## Validation
Unit tests with mocked Facebook API verify correct photo upload and post creation; multi-image posts work correctly; captions are included; API errors are handled; page token is used correctly.