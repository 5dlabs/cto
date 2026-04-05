Implement subtask 6003: Implement S3/R2 photo storage integration service

## Objective
Build an Effect.Service for uploading, retrieving, and managing photos in S3/R2 object storage. Handle image upload processing, generate storage keys, and provide CDN-ready URLs.

## Steps
1. Create src/services/storage.ts.
2. Define Effect.Service `StorageService` with methods: uploadImage(file: Buffer, metadata: ImageMetadata) -> Effect<StoredImage>, getImageUrl(fileKey: string) -> Effect<string>, deleteImage(fileKey: string) -> Effect<void>, listImages(prefix: string) -> Effect<StoredImage[]>.
3. Implement using @aws-sdk/client-s3 configured for R2 compatibility (custom endpoint).
4. Generate storage keys with pattern: `uploads/{year}/{month}/{uuid}.{ext}`.
5. On upload: use `sharp` to extract dimensions, generate a thumbnail variant (stored as `thumbnails/{key}`), and return metadata.
6. CDN URL generation: construct public URL from configured CDN domain + file key.
7. Implement as Effect Layer with configurable S3 endpoint, bucket, and credentials.
8. Handle errors: upload failures, invalid image formats, S3 connectivity issues.

## Validation
Unit tests with mocked S3 client verify correct key generation, thumbnail creation, and URL construction; integration test uploads a real image to a test bucket and verifies it's retrievable; error scenarios (invalid image, S3 down) return typed Effect errors.