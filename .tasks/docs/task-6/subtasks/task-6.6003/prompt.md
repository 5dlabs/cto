Implement subtask 6003: Implement image upload endpoint with S3/R2 storage

## Objective
Build the POST /api/v1/social/upload endpoint that accepts image files, stores them in S3/R2, and creates upload records in PostgreSQL.

## Steps
1. In `src/routes/upload.ts`, implement POST /api/v1/social/upload that: a) Accepts multipart/form-data with one or more image files. b) Validates file types (JPEG, PNG, WebP) and size limits. c) Generates unique S3 keys with organized path structure (e.g., uploads/2024/01/uuid.jpg). d) Uploads each file to S3/R2 via the storage Effect.Layer. e) Inserts Upload records into PostgreSQL. f) Returns 201 with array of upload IDs and metadata. 2. Validate request with Effect.Schema. 3. Handle errors: unsupported file type (400), file too large (413), storage failure (502). 4. Wire route into Elysia app.

## Validation
POST /upload with valid image returns 201 with upload ID; file exists in S3/R2 at expected key; Upload record exists in DB; invalid file type returns 400; oversized file returns 413; Effect.Schema validation rejects malformed requests.