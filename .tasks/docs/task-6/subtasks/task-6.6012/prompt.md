Implement subtask 6012: Implement portfolio sync to website

## Objective
Build the service that syncs published social media posts to the company website portfolio, ensuring published content is reflected on the website gallery.

## Steps
1. In `src/services/portfolio-sync.ts`, create an Effect.Service `PortfolioSyncService`. 2. Implement `syncToPortfolio(publishedPost: PublishedPost, draft: Draft) -> Effect<void>`: a) Determine the website portfolio API endpoint (from config). b) Prepare portfolio entry: images (from S3/R2 with public URLs or CDN URLs), caption, platform links, published date. c) POST the portfolio entry to the website CMS/API. d) Handle failures with retry logic. 3. Trigger portfolio sync automatically after successful publish (in the publish endpoint flow). 4. Add a manual re-sync endpoint `POST /api/v1/social/portfolio/sync/:post_id` for retrying failed syncs. 5. Track sync status on PublishedPost (synced_to_portfolio boolean).

## Validation
After publishing, portfolio sync is triggered and website API receives correct payload; failed syncs can be retried manually; sync status is tracked in DB; mock website API verifies correct data format.