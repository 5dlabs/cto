Implement subtask 6002: Implement image upload endpoint with S3/R2 storage

## Objective
Build the POST /api/v1/social/upload endpoint that accepts image files, stores them in S3/R2, and returns the stored URLs.

## Steps
1. Implement POST /api/v1/social/upload as a multipart form handler in Elysia. 2. Accept one or more image files (JPEG, PNG, WebP) with a max size of 10MB per file. 3. Validate file types and sizes using Effect.Schema. 4. Generate unique S3 keys using format: social/{year}/{month}/{uuid}.{ext}. 5. Upload each file to S3/R2 using the configured client with appropriate content-type headers. 6. Return a JSON response with an array of { url, key, size, content_type } for each uploaded file. 7. Handle upload failures gracefully — if one file in a batch fails, report which succeeded and which failed. 8. Wrap all S3 operations in Effect for structured error handling.

## Validation
Upload single and multiple images; verify files are accessible in S3/R2; reject oversized files (>10MB); reject non-image MIME types; verify returned URLs are valid and accessible.