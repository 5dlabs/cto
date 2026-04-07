Implement subtask 6012: Implement FacebookService with Effect retry and Graph API integration

## Objective
Implement the FacebookService Effect.Service that publishes posts to Facebook via the Graph API /photos or /feed endpoint with exponential backoff retry and timeout.

## Steps
In src/services/facebook.ts: implement FacebookService. publishPost(draft): POST to https://graph.facebook.com/v18.0/{FACEBOOK_PAGE_ID}/photos with params url=instagram_crop_url (reuse Instagram crop), caption=caption, access_token=FACEBOOK_PAGE_ACCESS_TOKEN. Returns {id, post_id}. Apply Effect.retry + Effect.timeout. On success insert social_posts with platform='facebook', external_post_id=post_id.

## Validation
Unit test: mock Graph API /photos returning {id:'123',post_id:'456'}. Verify social_posts row with platform='facebook', external_post_id='456', status='published'. Test timeout: mock API delays 35 seconds → Effect.timeout fires at 30s and result is PlatformError.