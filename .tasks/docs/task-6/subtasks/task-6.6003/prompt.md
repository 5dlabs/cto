Implement subtask 6003: Implement photo upload endpoint with multipart handling and R2 storage

## Objective
Build the POST /api/v1/social/upload endpoint that accepts multipart form data with photos, uploads them to R2, extracts image dimensions, and stores metadata in the uploads and photos tables.

## Steps
1. Create `src/routes/upload.ts` with Elysia route group.
2. `POST /api/v1/social/upload`:
   - Accept multipart form: `event_name` (string, required), `uploaded_by` (string, required), `photos` (File[], required, max 50 files).
   - Validate with Effect Schema: reject if no photos, if event_name is empty, if files exceed 20MB each.
   - Use Effect.gen to orchestrate the pipeline:
     a. Insert record into `uploads` table with event_name, uploaded_by, photo_count = files.length.
     b. For each photo file in parallel (Effect.forEach with concurrency: 5):
        - Read file buffer.
        - Use `sharp` to extract width/height metadata.
        - Generate R2 key via R2Service.generateKey.
        - Upload to R2 via R2Service.upload.
        - Insert record into `photos` table with upload_id, r2_key, original_filename, width, height.
     c. Return { upload_id, photo_count, photos: [{id, r2_key, width, height}] }.
3. Install `sharp` for image metadata extraction.
4. Error handling: If any photo upload fails, use Effect.catchTag to log the error and continue with remaining photos. Return partial success with list of failed filenames.
5. Response schema validation with Effect Schema for type-safe responses.

## Validation
Integration test: POST multipart form with 3 test JPEG images. Verify upload record created in DB with correct photo_count. Verify 3 photo records with correct dimensions, r2_keys. Mock R2Service to verify upload called 3 times. Test error case: one file is corrupted, verify partial success response with 2 photos and 1 failure listed.