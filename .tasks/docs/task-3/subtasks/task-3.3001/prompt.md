Implement subtask 3001: Implement MinIO S3 client with multipart upload and object tagging

## Objective
Create `src/modules/hermes/artifacts/minio-client.ts` — an S3-compatible client using `@aws-sdk/client-s3` configured from environment variables, supporting multipart upload for large images, content-type headers, object tagging, and presigned URL generation.

## Steps
1. Install `@aws-sdk/client-s3` and `@aws-sdk/s3-request-presigner` via Bun.
2. Create `minio-client.ts` that reads `MINIO_HERMES_ENDPOINT`, `MINIO_HERMES_BUCKET`, `MINIO_ACCESS_KEY`, `MINIO_SECRET_KEY` from environment (sourced from mounted secrets via `envFrom` on the ConfigMap).
3. Implement `uploadArtifact(key: string, body: Buffer, contentType: string, tags: Record<string, string>): Promise<UploadResult>` — uses `PutObjectCommand` for files ≤ 5MB, switches to `Upload` (from `@aws-sdk/lib-storage`) for multipart when > 5MB. Sets `ContentType` header and applies `Tagging` string (e.g., `retention-class=hermes&environment=${ENVIRONMENT}`).
4. Implement `generatePresignedUrl(key: string, ttlSeconds: number): Promise<string>` using `getSignedUrl` from `@aws-sdk/s3-request-presigner`. Default TTL 3600s, configurable via `PRESIGNED_URL_TTL_SECONDS` env var.
5. Implement `deleteArtifact(key: string): Promise<void>` for cleanup scenarios.
6. Export typed interfaces: `UploadResult { key: string; bucket: string; size: number; etag: string }`.
7. Add connection validation on startup — call `HeadBucket` to confirm bucket exists.

## Validation
Unit test: mock S3 client and verify `uploadArtifact` calls `PutObjectCommand` for a 1MB buffer and uses multipart for a 6MB buffer. Integration test: upload a test PNG to the MinIO dev bucket, verify it exists via `HeadObject`, verify object tags via `GetObjectTagging`, generate a presigned URL and confirm HTTP GET returns 200 with `content-type: image/png`. Verify `HeadBucket` is called during client initialization.