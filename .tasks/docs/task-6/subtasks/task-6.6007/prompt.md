Implement subtask 6007: Implement Effect.Services for LinkedIn publishing

## Objective
Build an Effect.Service for publishing content to LinkedIn via the LinkedIn Marketing API, including image upload, article/post creation, and status tracking.

## Steps
1. Create src/integrations/linkedin.ts.
2. Define Effect.Service `LinkedInPublisher` with methods: publish(content: PublishableContent) -> Effect<PublishResult>, getPostStatus(postId: string) -> Effect<PostStatus>.
3. Implement LinkedIn UGC (User Generated Content) Post API: register image upload -> upload binary -> create post with image URN.
4. Support organization posts (posting as company page).
5. Handle LinkedIn OAuth2 token refresh flow.
6. Map content to LinkedIn's specific format: text with media references.
7. Handle API errors and rate limits.
8. Implement mock for testing.

## Validation
Unit tests with mocked LinkedIn API verify correct three-step flow (register -> upload -> post); organization vs personal posting uses correct author URN; OAuth refresh triggered appropriately; error mapping works correctly.