Implement subtask 6007: Implement AI caption generation and draft creation pipeline

## Objective
Build the caption generation service using OpenAI/Claude that creates platform-specific captions with appropriate tone, hashtags, and formatting. Wire the full AI curation pipeline (score → select → crop → caption → create drafts) triggered after upload.

## Steps
1. Create `src/services/CaptionService.ts` as an Effect.Service:
   - Method `generateCaption(platform: Platform, eventName: string, photoDescriptions: string[], brandContext?: string)`: Effect<CaptionResult, CaptionError>
     - Call OpenAI chat completion with platform-specific system prompts:
       - Instagram: Casual, emoji-rich, 2200 char max, 20-30 hashtags.
       - LinkedIn: Professional, thought-leadership tone, 3000 char max, 3-5 hashtags.
       - Facebook: Conversational, community-focused, 500 char max, 5-10 hashtags.
       - TikTok: Trendy, short, 150 char max, 5-8 trending hashtags.
     - Return { caption: string, hashtags: string[] }.
   - Wrap in Effect.retry with Schedule.exponential.
2. Create `src/pipelines/CurationPipeline.ts` — orchestrates the full flow:
   - Input: upload_id (triggered after upload completes).
   - Step 1: Fetch all photos for upload_id from DB.
   - Step 2: Score photos via AIScoringService.scoreAndSelectPhotos.
   - Step 3: For each target platform (instagram, linkedin, facebook, tiktok):
     a. Generate crops via ImageCropService.generateCropsForDrafts.
     b. Generate caption via CaptionService.generateCaption.
     c. Insert draft record: upload_id, platform, caption, hashtags, image_keys (cropped R2 keys), status 'draft'.
   - Step 4: Return { drafts_created: number, platforms: string[] }.
3. Wire CurationPipeline into the upload endpoint (6003): after successful upload, trigger pipeline asynchronously using Effect.fork (fire-and-forget with error logging).
4. Add a manual re-trigger endpoint: `POST /api/v1/social/uploads/:id/curate` — re-runs the curation pipeline for an existing upload.
5. Create `src/services/CaptionService.live.ts` — Effect Layer.

## Validation
Unit test: (1) Mock OpenAI, verify Instagram caption includes emojis and 20+ hashtags. (2) Verify LinkedIn caption is professional tone with 3-5 hashtags. (3) Verify TikTok caption under 150 chars. Integration test: (4) Upload 10 test photos, run CurationPipeline, verify 4 draft records created (one per platform) with correct image_keys and captions. (5) Verify re-trigger endpoint creates new drafts (or updates existing). (6) Test pipeline error handling: if caption generation fails for one platform, other platforms' drafts still created.