Implement subtask 6007: Implement website portfolio sync via webhook/API

## Objective
Build the webhook/API integration that syncs published social media content to the company website portfolio.

## Steps
1. Create `src/services/portfolio-sync.ts` module.
2. Define a `PortfolioSyncService` Effect.Service with method: `syncPost(publishedPost: PublishedPost) -> Effect.Effect<SyncResult, SyncError>`.
3. Implement the sync logic:
   - After successful publishing, prepare a payload with: image URL (S3 presigned or public), caption, platform, published date, external post URL.
   - Send the payload to the website's portfolio API endpoint (configurable URL from environment).
   - Alternatively, emit a webhook to a configured URL with the same payload.
   - Update `published_posts.sync_status` to 'synced' or 'failed'.
4. Implement a retry mechanism for failed syncs using Effect.Schedule.
5. Implement GET `/api/v1/social/sync/status` — list sync status of published posts.
6. Implement POST `/api/v1/social/sync/retry/:postId` — manually retry a failed sync.
7. Hook the sync into the publishing flow so it triggers automatically after successful publish.

## Validation
Mock the website portfolio API. Verify sync is triggered after publishing. Verify sync payload contains correct data. Test retry on failed sync. Verify sync_status is updated correctly. Test manual retry endpoint.