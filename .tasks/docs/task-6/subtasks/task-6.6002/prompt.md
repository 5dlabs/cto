Implement subtask 6002: Implement S3/R2 photo upload and storage service

## Objective
Build the photo upload and storage module using S3-compatible API (Cloudflare R2 or AWS S3 per dp-4), handling multipart uploads, generating signed URLs for retrieval, and wiring into the /api/v1/social/upload endpoint.

## Steps
1. Create a `services/storage.ts` module. 2. Use @aws-sdk/client-s3 (S3-compatible, works with both R2 and S3). 3. Configure the client from environment variables: endpoint, access_key_id, secret_access_key, bucket_name, region. 4. Implement `uploadPhotos(files: File[], eventName: string): Effect.Effect<PhotoReference[], StorageError>` — uploads each file with a unique key (e.g., `events/{eventName}/{uuid}.{ext}`), returns array of { key, url, size, mimeType }. 5. Implement `getSignedUrl(key: string): Effect.Effect<string, StorageError>` for temporary read access. 6. Wire into the POST /api/v1/social/upload endpoint: accept multipart form data with event_name and photos[], upload to storage, create a new Draft record with status 'pending_curation', return the draft with photo references. 7. Add Effect.retry with exponential backoff for upload failures. 8. Validate file types (JPEG, PNG, WEBP only) and size limits (max 20MB per file).

## Validation
Upload endpoint accepts multipart form data with photos and returns a draft with photo references. File type validation rejects non-image files. Signed URLs are generated and accessible. Effect.retry retries on transient S3 errors. A Draft record is persisted in PostgreSQL with status 'pending_curation'.