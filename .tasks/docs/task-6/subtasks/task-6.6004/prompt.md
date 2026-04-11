<identity>
You are nova working on subtask 6004 of task 6.
</identity>

<context>
<scope>
Implement POST /api/v1/social/upload as a multipart Elysia route that accepts JPEG/PNG files, uploads them to Cloudflare R2 via the S3-compatible SDK, creates a social_drafts row with status=pending_curation, and triggers the async curation pipeline.
</scope>
</context>

<implementation_plan>
Add Elysia multipart body parsing. In the handler: validate content-type is multipart/form-data, extract files array and event_id. For each file: use @aws-sdk/client-s3 PutObjectCommand with bucket=R2_BUCKET_NAME, key=`events/{event_id}/source/{uuid}.jpg`, endpoint=R2_ENDPOINT, credentials from R2_ACCESS_KEY_ID/R2_SECRET_ACCESS_KEY. Collect uploaded URLs. Insert social_drafts row with source_photo_urls=[...urls], status='pending_curation'. Call `void runCurationPipeline(draft_id, urls)` as a fire-and-forget async call (do not await). Return 202 {draft_id, status:'pending_curation'}. Implement GET /api/v1/social/drafts and GET /api/v1/social/drafts/:id querying the social_drafts table.
</implementation_plan>

<validation>
POST /api/v1/social/upload with 3 JPEG files returns 202 with a valid UUID draft_id and status=pending_curation within 2 seconds. Verify the 3 source files exist in R2 bucket under the correct key prefix. GET /api/v1/social/drafts/:id returns the draft row with source_photo_urls containing 3 URLs.
</validation>