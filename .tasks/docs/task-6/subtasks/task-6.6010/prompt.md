Implement subtask 6010: Implement LinkedInService with Effect retry and Share API integration

## Objective
Implement the LinkedInService Effect.Service that publishes posts to LinkedIn via the UGC Posts Share API with exponential backoff retry and timeout.

## Steps
In src/services/linkedin.ts: implement LinkedInService. publishPost(draft): POST to https://api.linkedin.com/v2/ugcPosts with body {author: 'urn:li:organization:{LINKEDIN_ORG_ID}', lifecycleState:'PUBLISHED', specificContent:{shareCommentary:{text:caption}, shareMediaCategory:'IMAGE', media:[{status:'READY',originalUrl:linkedin_crop_url}]}, visibility:{memberNetworkVisibility:'PUBLIC'}}. Set Authorization: Bearer {LINKEDIN_ACCESS_TOKEN}. Apply same Effect.retry + Effect.timeout pattern as Instagram. On success, insert social_posts row with platform='linkedin'.

## Validation
Unit test: mock LinkedIn API returning 201 on first attempt. Verify social_posts row inserted with platform='linkedin' and status='published'. Mock 429 rate limit for 3 attempts then 201: verify retry behavior and final success. Verify Authorization header is set correctly in mock.