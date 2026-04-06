Implement subtask 6006: Implement LinkedIn API publishing integration

## Objective
Build the LinkedIn publishing client that posts approved content to a LinkedIn company page via the LinkedIn API.

## Steps
1. Create `src/publishing/linkedin.ts` module.
2. Implement LinkedIn Share API integration:
   - Register image upload → create share with image and text.
   - Handle OAuth2 authentication with bearer token.
   - Read LINKEDIN_ACCESS_TOKEN and LINKEDIN_ORG_ID from Kubernetes secrets.
3. Implement `publishToLinkedIn(imageUrl: string, caption: string) -> Effect<PublishResult>` returning the share URN and URL.
4. Handle API rate limits, token refresh, and error responses.
5. Define Effect.Schema types for PublishResult.
6. Add unit tests with mocked LinkedIn API responses.

## Validation
Unit tests with mocked LinkedIn API verify: successful share creation returns URN; image upload flow completes before share creation; authentication errors are handled; caption is properly formatted for LinkedIn (no hashtag overload).