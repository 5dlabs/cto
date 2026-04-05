Implement subtask 6008: Implement Facebook API publishing integration

## Objective
Build the Facebook publishing module to post approved content to a Facebook page using the Facebook Graph API.

## Steps
1. Create a `services/publishers/facebook.ts` module implementing the `Publisher` interface. 2. Implement Facebook Graph API integration: a) For single photo: POST /{page-id}/photos with source (image URL) and caption. b) For multiple photos: upload each photo unpublished, then create a multi-photo post. 3. Manage Facebook Page access token (long-lived page token that doesn't expire, obtained from user token exchange). 4. Handle: rate limits, content moderation flags, image format restrictions. 5. On success, create a `PublishedPost` record with platform='facebook' and the Facebook post ID. 6. Use Effect.retry for transient failures.

## Validation
With valid Facebook page token and an approved draft, the publisher posts photos with caption to the page. Multi-photo posts work correctly. PublishedPost record is created with Facebook post ID. Rate limits trigger retry. Content moderation errors are surfaced.