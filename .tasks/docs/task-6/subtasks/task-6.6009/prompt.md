Implement subtask 6009: Implement LinkedIn publishing integration

## Objective
Create an Effect.Service for the LinkedIn API integration to publish approved posts to a LinkedIn profile or company page.

## Steps
1. Create `src/services/LinkedInService.ts` as an Effect.Service tag with methods: publishPost(imageUrl: string, caption: string, accessToken: string) → Effect<PublishResult, PlatformPublishError>, getPostAnalytics(postUrn: string, accessToken: string) → Effect<Engagement, PlatformPublishError>. 2. LinkedIn API flow: POST to /v2/assets?action=registerUpload to get upload URL → PUT binary image to upload URL → POST to /v2/ugcPosts with image asset URN and caption (ShareCommentary). 3. Handle LinkedIn OAuth2 token refresh via refresh_token grant. 4. Handle LinkedIn-specific constraints: caption max 3000 chars, image must be JPEG/PNG, max 10MB. 5. Map LinkedIn post URN to platformPostId in PublishResult. 6. Wrap all HTTP calls in Effect.tryPromise with LinkedInPublishError, LinkedInAuthError tags.

## Validation
Mock LinkedIn API and verify three-step publish flow (register upload, upload binary, create UGC post). Verify token refresh on 401 response. Verify image format validation rejects non-JPEG/PNG. Verify caption length validation.