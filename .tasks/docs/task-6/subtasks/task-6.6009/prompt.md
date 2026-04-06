Implement subtask 6009: Implement InstagramService with Effect retry and Graph API integration

## Objective
Implement the InstagramService Effect.Service that uploads media and publishes to Instagram via the Graph API, with exponential backoff retry (max 5 attempts) and 30-second timeout per call.

## Steps
In src/services/instagram.ts: implement InstagramService using Effect.Service. uploadMedia(image_url): POST to https://graph.facebook.com/v18.0/{INSTAGRAM_BUSINESS_ACCOUNT_ID}/media with params image_url, caption, access_token=INSTAGRAM_ACCESS_TOKEN. Returns {id: creation_id}. publishPost(creation_id): POST to /media_publish with creation_id. Wrap each HTTP call in Effect.tryPromise, compose with Effect.retry(Schedule.exponential('1 second').pipe(Schedule.compose(Schedule.recurs(5)))), Effect.timeout('30 seconds'). On final failure after retries, return Effect.fail(new PlatformError({platform:'instagram', message})). Insert social_posts row on success with platform='instagram', external_post_id, status='published'.

## Validation
Unit test with vitest mock: mock Graph API to fail twice then succeed. Run Effect.runPromise. Verify the HTTP call was made 3 times total (2 failures + 1 success) and final result contains external_post_id. Mock permanent failure (5 retries): verify Effect fails with PlatformError and social_posts row has status='failed'.