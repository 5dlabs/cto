Implement subtask 6003: Implement S3/R2 object storage layer with Effect

## Objective
Create an Effect Layer for S3-compatible object storage (R2 or S3) for photo uploads, thumbnail generation, and retrieval. Read bucket configuration from ConfigMap endpoints.

## Steps
1. Install `@aws-sdk/client-s3` and `@aws-sdk/s3-request-presigner`. 2. Create `src/layers/StorageLayer.ts` as an Effect.Layer reading S3_ENDPOINT, S3_BUCKET, S3_ACCESS_KEY_ID, S3_SECRET_ACCESS_KEY from env. 3. Create `src/services/StorageService.ts` as an Effect.Service tag with methods: uploadPhoto(buffer, key, contentType) → Effect<string, StorageError>, getPresignedUrl(key, expiresIn) → Effect<string, StorageError>, deletePhoto(key) → Effect<void, StorageError>, listPhotos(prefix) → Effect<string[], StorageError>. 4. Implement key naming convention: `social/originals/{uuid}.{ext}` and `social/thumbnails/{uuid}.{ext}`. 5. Use Effect.tryPromise to wrap AWS SDK calls with proper error tagging (StorageUploadError, StorageNotFoundError). 6. Add storage connectivity check to the health endpoint.

## Validation
Upload a test image buffer and verify it returns a valid S3 key. Generate a presigned URL and verify it is well-formed. Delete the test object and confirm it no longer exists. Health check reports storage status.