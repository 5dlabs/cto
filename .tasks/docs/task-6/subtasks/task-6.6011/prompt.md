<identity>
You are nova working on subtask 6011 of task 6.
</identity>

<context>
<scope>
Implement the TikTokService Effect.Service that uploads video/photo content to TikTok via the Content Posting API with exponential backoff retry and timeout.
</scope>
</context>

<implementation_plan>
In src/services/tiktok.ts: implement TikTokService. uploadVideo(draft): POST to https://open.tiktokapis.com/v2/post/publish/video/init/ with Authorization: Bearer {TIKTOK_ACCESS_TOKEN}, body {post_info:{title:caption,privacy_level:'PUBLIC_TO_EVERYONE'}, source_info:{source:'PULL_FROM_URL', video_url:tiktok_crop_url}}. Poll for publish_id status if needed. Apply Effect.retry + Effect.timeout. On success insert social_posts with platform='tiktok'. If TIKTOK_ACCESS_TOKEN env var is not set, return Effect.fail(new PlatformError({platform:'tiktok', message:'TikTok not configured'})) immediately without retrying.
</implementation_plan>

<validation>
Unit test: mock Content Posting API returning success. Verify social_posts row with platform='tiktok' and status='published'. Test unconfigured path: TIKTOK_ACCESS_TOKEN not set → immediate Effect.fail with no retry attempts. Test retry: 2 failures + 1 success → 3 HTTP calls total.
</validation>