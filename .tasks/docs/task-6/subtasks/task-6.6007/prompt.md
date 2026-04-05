Implement subtask 6007: Implement LinkedIn API publishing integration

## Objective
Build the LinkedIn publishing module using the LinkedIn Marketing API to publish approved images with professional captions to the company page.

## Steps
1. Create a `services/publishing/linkedin` module implementing the PublishingProvider interface. 2. Implement LinkedIn API integration using the Share API (v2): register image upload → upload image binary → create share with image and caption. 3. Use OAuth2 tokens stored in Kubernetes secrets. Implement refresh token flow. 4. Format caption for LinkedIn: professional tone, no hashtags in excess, include relevant mentions if configured. 5. Return PublishResult with platform_post_id (activityUrn), share URL, published_at. 6. Handle LinkedIn-specific errors: media upload failures, permission issues, rate limits (daily share limits).

## Validation
Unit test with mocked LinkedIn API: verify three-step upload/share flow, OAuth token refresh, rate limit handling. Verify caption formatting meets LinkedIn requirements (max 3000 chars). Integration test with LinkedIn test organization if available.