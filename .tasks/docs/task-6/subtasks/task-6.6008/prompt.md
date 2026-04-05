Implement subtask 6008: Implement Facebook API publishing integration

## Objective
Build the Facebook publishing module using the Facebook Graph API to publish approved images with captions to the business page.

## Steps
1. Create a `services/publishing/facebook` module implementing the PublishingProvider interface. 2. Implement Facebook Graph API integration: POST to /{page-id}/photos with image URL and caption. 3. Use Page Access Token stored in Kubernetes secrets. Implement token validation and refresh. 4. Return PublishResult with platform_post_id (post_id), permalink_url, published_at. 5. Handle Facebook-specific errors: invalid page token, image format issues, posting rate limits. 6. Support both photo posts and link posts with image previews.

## Validation
Unit test with mocked Facebook Graph API: verify photo post creation, token validation, error handling for invalid tokens. Verify returned post_id format is valid. Integration test with Facebook test page if available.