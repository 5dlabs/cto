Implement subtask 6004: Implement photo upload endpoint and storage pipeline

## Objective
Build the POST /api/v1/social/upload endpoint that accepts photo files, stores originals in S3/R2, generates thumbnails, and persists metadata to PostgreSQL.

## Steps
1. Create `src/routes/upload.ts` with POST /api/v1/social/upload accepting multipart/form-data with file field. 2. Define Effect.Schema for upload request validation (file type must be image/jpeg, image/png, image/webp; max size 20MB) and response schema (id, s3_key, thumbnail_url, uploaded_at). 3. On upload: generate UUID, upload original to StorageService at `social/originals/{uuid}.{ext}`. 4. Use `sharp` library (install it) to generate a thumbnail (800x800 max, preserve aspect ratio), upload to `social/thumbnails/{uuid}.{ext}`. 5. Insert record into `social_photos` table with original key, thumbnail key, file metadata (dimensions, size, format extracted via sharp). 6. Return the created photo record with presigned URLs for both original and thumbnail. 7. Wrap entire pipeline in Effect.gen with proper error handling: StorageError, DatabaseError, ValidationError. 8. Add Prometheus counter for uploads (social_photos_uploaded_total).

## Validation
Upload a valid JPEG and verify 201 response with photo ID, S3 keys, and presigned URLs. Upload an invalid file type and verify 400 validation error. Upload a file exceeding 20MB and verify rejection. Verify database record matches returned data.