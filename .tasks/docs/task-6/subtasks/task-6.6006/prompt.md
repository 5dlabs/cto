Implement subtask 6006: Implement Instagram API publishing integration

## Objective
Build the Instagram publishing module using the Instagram Graph API to publish approved images with captions.

## Steps
1. Create a `services/publishing/instagram` module. 2. Define a PublishingProvider interface with Effect: `publish(imageUrl: string, caption: string) -> Effect<PublishResult, PublishError>`. 3. Implement Instagram Graph API integration: use the Content Publishing API (requires Facebook Page linked to Instagram Professional account). 4. Flow: POST image URL to /media endpoint to create a container → POST container ID to /media_publish endpoint to publish. 5. Store Instagram access token (long-lived) in Kubernetes secrets. Implement token refresh logic (tokens expire every 60 days). 6. Return PublishResult with platform_post_id, permalink, published_at. 7. Handle API errors: rate limits (respect X-Business-Use-Case-Usage headers), invalid media, expired token.

## Validation
Unit test with mocked Instagram API: verify two-step publish flow, token refresh logic, rate limit handling. Integration test with Instagram sandbox/test account if available. Verify PublishResult contains valid permalink.