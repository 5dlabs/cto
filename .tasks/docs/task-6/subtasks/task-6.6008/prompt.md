Implement subtask 6008: Implement AI curation pipeline orchestrating scoring, cropping, and captioning

## Objective
Build the end-to-end AI curation pipeline that is triggered after image upload: score images via ImageCurationService, select top images, generate platform crops via CropService, generate captions via CaptionService, and create a Draft record with status 'pending_approval'.

## Steps
1. Create `src/pipelines/CurationPipeline.ts`.
2. Define `runCurationPipeline(uploadIds: string[], eventId?: string, platforms: string[]): Effect.Effect<Draft, CurationPipelineError>`:
   a. Fetch upload records from database by IDs.
   b. Call `ImageCurationService.scoreImages()` with all upload image keys.
   c. Call `ImageCurationService.selectTopImages()` to pick top 5-10.
   d. For each selected image, call `CropService.generateCrops()` for all target platforms — use `Effect.forEach` with bounded concurrency.
   e. Call `CaptionService.generateCaption()` with event context and image descriptions from scoring step.
   f. Insert a `drafts` row: `upload_ids` = selected image IDs, `caption`, `hashtags`, `platforms`, `status = 'pending_approval'`, `platform_crops` = aggregated crops JSON, `ai_score` = average overall score.
   g. Return the created Draft.
3. Define `CurationPipelineError` that wraps sub-service errors with context.
4. Wire the pipeline invocation from the upload endpoint (subtask 6004) — call `Effect.runFork` so it runs asynchronously after upload response is sent.
5. Handle partial failures gracefully: if cropping fails for one image, continue with others. If captioning fails, create draft without caption (status still pending_approval, caption can be manually added).

## Validation
Integration test: upload 10 images → verify pipeline runs and creates a draft with status 'pending_approval', upload_ids containing top 5 scored images, platform_crops populated for all platforms, caption and hashtags populated. Test partial failure: mock CropService to fail for 1 image → verify draft is still created with crops for remaining images.