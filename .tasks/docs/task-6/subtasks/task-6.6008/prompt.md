Implement subtask 6008: Implement LinkedIn publishing Effect.Service

## Objective
Build the Effect.Service implementation for publishing posts to LinkedIn, including the LinkedIn Marketing API integration for company page posts.

## Steps
1. In `src/services/publishers/linkedin.ts`, create an Effect.Service `LinkedInPublisher` implementing the `Publisher` interface. 2. Implement `publish(draft: Draft, images: Upload[]) -> Effect<PublishResult>`: a) Register image upload with LinkedIn API. b) Upload image binary to the provided upload URL. c) Create a share/post on the company page with image asset, caption text, and LinkedIn-appropriate formatting. d) Return PublishResult with platform_post_id and post_url. 3. Handle LinkedIn OAuth2 token management. 4. Handle LinkedIn API rate limits and error responses. 5. Format captions appropriately for LinkedIn (professional tone, no hashtag overload).

## Validation
Unit tests with mocked LinkedIn API verify correct three-step upload flow (register → upload → share); post is created on correct company page; captions are LinkedIn-formatted; API errors are handled; OAuth token refresh works.