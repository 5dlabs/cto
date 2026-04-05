Implement subtask 6003: Implement R2 storage integration via @aws-sdk/client-s3

## Objective
Create an R2StorageService as an Effect service that handles uploading files to Cloudflare R2 (S3-compatible) under the `social/` prefix, generating presigned URLs for retrieval, and deleting objects.

## Steps
1. Install `@aws-sdk/client-s3` and `@aws-sdk/s3-request-presigner`.
2. Create `src/services/R2StorageService.ts` as an Effect.Service.
3. Configure S3Client with R2 endpoint, access key, secret key, and bucket name from environment variables (R2_ENDPOINT, R2_ACCESS_KEY_ID, R2_SECRET_ACCESS_KEY, R2_BUCKET).
4. Implement methods:
   - `upload(key: string, body: Buffer | ReadableStream, contentType: string): Effect.Effect<string, R2Error>` — uploads to `social/{key}`, returns the full R2 key.
   - `getPresignedUrl(key: string, expiresIn?: number): Effect.Effect<string, R2Error>` — returns a presigned GET URL, default 1 hour expiry.
   - `delete(key: string): Effect.Effect<void, R2Error>` — deletes the object.
5. Define `R2Error` as a tagged Effect error with context.
6. Create the Effect Layer `R2StorageServiceLive` that provides the S3Client configuration.
7. Export the service tag and layer for dependency injection.

## Validation
Unit test with mocked S3Client: verify `upload` calls PutObjectCommand with correct bucket and key prefix. Verify `getPresignedUrl` returns a URL string. Verify `delete` calls DeleteObjectCommand. Test that R2Error is properly tagged on S3 SDK failures.