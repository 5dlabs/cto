Implement subtask 6007: Implement Facebook API publishing integration

## Objective
Build the Facebook publishing client that posts approved content to a Facebook page via the Graph API.

## Steps
1. Create `src/publishing/facebook.ts` module.
2. Implement Facebook Graph API integration:
   - Upload photo to page → create post with photo and message.
   - Handle Page Access Token authentication.
   - Read FACEBOOK_PAGE_ACCESS_TOKEN and FACEBOOK_PAGE_ID from Kubernetes secrets.
3. Implement `publishToFacebook(imageUrl: string, caption: string) -> Effect<PublishResult>` returning the post ID and URL.
4. Handle API rate limits, token expiration, and error responses.
5. Define Effect.Schema types for PublishResult.
6. Add unit tests with mocked Facebook API responses.

## Validation
Unit tests with mocked Facebook Graph API verify: successful photo post returns post ID; page token errors are handled; rate limits produce appropriate retry behavior; caption is properly formatted for Facebook.