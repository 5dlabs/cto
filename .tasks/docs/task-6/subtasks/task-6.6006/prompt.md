Implement subtask 6006: Implement Instagram API publishing integration

## Objective
Build the Instagram publishing module to post approved content (photos + captions) to Instagram using the Instagram Graph API.

## Steps
1. Create a `services/publishers/instagram.ts` module implementing a `Publisher` interface: `publish(draft: Draft): Effect.Effect<PublishResult, PublishError>`. 2. Implement Instagram Graph API integration: a) Use the Content Publishing API: create media container (POST /{ig-user-id}/media with image_url and caption), then publish (POST /{ig-user-id}/media_publish). b) Handle carousel posts for multiple selected photos. 3. Manage Instagram OAuth token: store long-lived token, implement token refresh before expiry. 4. Handle: rate limits (200 calls/hour), content policy errors, media upload failures. 5. On success, create a `PublishedPost` record with platform='instagram' and the Instagram post ID. 6. Use Effect.retry for transient failures, Effect error channels for permanent failures (e.g., content policy violation).

## Validation
With a valid Instagram token and an approved draft, the publisher creates a media container and publishes successfully. PublishedPost record is created with Instagram post ID. Carousel posts work for multiple photos. Token refresh is triggered when token is near expiry. Rate limit errors trigger retry with backoff. Content policy violations are surfaced as permanent errors.