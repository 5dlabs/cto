Implement subtask 6007: Implement Instagram publishing Effect.Service

## Objective
Build the Effect.Service implementation for publishing posts to Instagram, including platform-specific image cropping and the Instagram Graph API integration.

## Steps
1. In `src/services/publishers/instagram.ts`, create an Effect.Service `InstagramPublisher` implementing a `Publisher` interface. 2. Implement `publish(draft: Draft, images: Upload[]) -> Effect<PublishResult>`: a) Crop/resize images to Instagram-compatible dimensions (1080x1080 square, 1080x1350 portrait, 1080x566 landscape) using sharp. b) Upload media to Instagram via Graph API container creation. c) Create the media publish request with caption and hashtags. d) Return PublishResult with platform_post_id and post_url. 3. Handle Instagram API rate limits and error codes. 4. Implement token refresh for long-lived Instagram tokens. 5. Add `sharp` as a dependency for image processing.

## Validation
Unit tests with mocked Instagram API verify correct API call sequence (container create → publish); images are resized to correct dimensions; captions and hashtags are included; API errors return proper error types; token refresh is triggered on 401.