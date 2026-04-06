Implement subtask 6002: Implement S3/R2 photo upload and storage integration

## Objective
Build the photo upload endpoint and S3/R2 storage integration for the /api/v1/social/upload endpoint, handling multipart file uploads and returning stored image URLs.

## Steps
1. Add @aws-sdk/client-s3 dependency (compatible with both S3 and R2 via S3-compatible API).
2. Create `src/storage/s3-client.ts` module:
   - Configure S3Client reading S3_ENDPOINT, S3_BUCKET, S3_ACCESS_KEY_ID, S3_SECRET_ACCESS_KEY from environment (Kubernetes secrets).
   - Implement `uploadImage(file: Buffer, filename: string, contentType: string) -> Effect<string>` returning the public/signed URL.
   - Implement `deleteImage(key: string) -> Effect<void>` for cleanup.
3. Implement POST `/api/v1/social/upload` Elysia handler:
   - Accept multipart/form-data with one or more image files.
   - Validate file types (JPEG, PNG, WebP only) and size limits (10MB per file).
   - Upload each file to S3/R2 with a unique key (e.g., `social/{uuid}/{filename}`).
   - Return array of uploaded image URLs.
4. Validate request/response with Effect.Schema.
5. Add error handling for storage failures.

## Validation
Unit tests with mocked S3 client verify upload/delete operations; integration test uploads a test image and receives a valid URL; invalid file types are rejected with 400; oversized files are rejected; multiple files upload concurrently.