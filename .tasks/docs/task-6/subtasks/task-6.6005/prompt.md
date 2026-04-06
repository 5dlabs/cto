Implement subtask 6005: Implement Instagram API publishing integration

## Objective
Build the Instagram publishing client that posts approved content to Instagram via the Instagram Graph API.

## Steps
1. Create `src/publishing/instagram.ts` module.
2. Implement Instagram Graph API integration:
   - Use the Content Publishing API flow: create media container → publish media.
   - Handle single image and carousel posts.
   - Read INSTAGRAM_ACCESS_TOKEN and INSTAGRAM_BUSINESS_ACCOUNT_ID from Kubernetes secrets.
3. Implement `publishToInstagram(imageUrl: string, caption: string) -> Effect<PublishResult>` returning the platform post ID and URL.
4. Handle API rate limits (200 calls/hour), token expiration, and error responses.
5. Define Effect.Schema types for PublishResult.
6. Add unit tests with mocked Instagram API responses.

## Validation
Unit tests with mocked Instagram API verify: successful publish returns post ID; rate limit error is handled with appropriate retry/backoff; expired token returns a clear error; caption with hashtags is properly formatted.