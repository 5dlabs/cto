Implement subtask 6002: Implement Cloudflare R2 integration service with S3-compatible client

## Objective
Build an Effect Service for Cloudflare R2 using @aws-sdk/client-s3 that handles photo uploads, deletion, and CDN URL generation. This service is used by the upload endpoint, AI pipeline, and GDPR deletion.

## Steps
1. Install `@aws-sdk/client-s3` package.
2. Create `src/services/R2Service.ts` as an Effect.Service tag:
   - Constructor takes R2_ACCOUNT_ID, R2_ACCESS_KEY_ID, R2_SECRET_ACCESS_KEY, R2_BUCKET_NAME, R2_PUBLIC_URL from environment.
   - Configure S3Client with endpoint `https://${R2_ACCOUNT_ID}.r2.cloudflarestorage.com`.
3. Methods (all returning Effect):
   - `upload(key: string, body: Buffer, contentType: string)`: PutObjectCommand, returns the r2_key.
   - `delete(key: string)`: DeleteObjectCommand.
   - `deleteBatch(keys: string[])`: DeleteObjectsCommand for bulk deletion (GDPR).
   - `getPublicUrl(key: string)`: Returns `${R2_PUBLIC_URL}/${key}`.
   - `generateKey(uploadId: string, filename: string)`: Returns structured key like `social/uploads/${uploadId}/${uuid}-${filename}`.
4. Create `src/services/R2Service.live.ts` — Effect Layer that constructs the live R2Service from config.
5. Wrap all AWS SDK calls in `Effect.tryPromise` with tagged errors (R2UploadError, R2DeleteError).
6. Add content-type detection for common image types (jpeg, png, webp, heic).

## Validation
Unit test with mocked S3Client: verify upload sends correct PutObjectCommand params, delete sends correct DeleteObjectCommand, deleteBatch handles multiple keys. Verify getPublicUrl returns correctly formatted URLs. Verify generateKey produces unique structured paths.