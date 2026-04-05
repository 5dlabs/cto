Implement subtask 6002: Implement photo upload endpoint and S3 storage

## Objective
Build the photo upload endpoint that accepts image files, stores them in S3-compatible object storage, and persists metadata in PostgreSQL.

## Steps
1. Create `src/routes/photos.ts` module.
2. Implement POST `/api/v1/social/photos/upload` endpoint:
   - Accept multipart/form-data with one or more image files.
   - Validate file types (JPEG, PNG, WebP) and size limits (e.g., 20MB per file).
   - Generate unique S3 keys with timestamp and UUID.
   - Upload each file to S3 using @aws-sdk/client-s3 PutObject.
   - Extract image metadata (dimensions, size, format) if feasible.
   - Insert a record into the `photos` table with s3_key, s3_url, and metadata.
   - Return the created photo records with presigned URLs for access.
3. Implement GET `/api/v1/social/photos` — list uploaded photos with pagination.
4. Implement GET `/api/v1/social/photos/:id` — get a single photo with presigned S3 URL.
5. Use Effect.Schema for request/response validation.
6. Add error handling for S3 upload failures, invalid files, and database errors.

## Validation
Upload a valid image and verify it appears in S3 (or mock) and the database. Attempt upload of invalid file type and verify 400 error. List photos and verify pagination. Retrieve a single photo and verify presigned URL is returned.