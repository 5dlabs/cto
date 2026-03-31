Implement subtask 3006: Implement presigned URL endpoint for artifact retrieval

## Objective
Add `GET /api/hermes/artifacts/:id/url` Elysia route that looks up an artifact record by ID, generates a presigned S3 URL for the corresponding MinIO object, and returns it to authenticated callers with the `hermes:read` permission.

## Steps
1. Add a new route in the Hermes Elysia router: `GET /api/hermes/artifacts/:id/url`.
2. Require `hermes:read` permission — use existing auth middleware/guard from the Hermes module.
3. Implementation flow:
   - Parse `:id` from route params, validate it's a valid artifact ID format
   - Fetch the artifact record via `ArtifactPersistence.getArtifactById(id)`
   - If not found, return 404 with structured error body `{ error: 'ARTIFACT_NOT_FOUND' }`
   - Call `MinioClient.generatePresignedUrl(artifact.storageKey, ttlSeconds)` where TTL comes from `PRESIGNED_URL_TTL_SECONDS` env var (default 3600)
   - Return `{ url: string; expiresAt: string; contentType: string; sizeBytes: number }`
4. Also add `GET /api/hermes/deliberations/:id/artifacts` to list all artifacts for a deliberation:
   - Fetch via `ArtifactPersistence.getArtifactsByDeliberation(deliberationId)`
   - Return array of artifact records (without presigned URLs — client calls the individual endpoint)
5. Add response type schemas using Elysia's type system for OpenAPI documentation.
6. Log each presigned URL generation with structured fields: `deliberation_id`, `artifact_id`, `storage_key`, `operation: 'presign'`, `duration_ms`.

## Validation
Integration test: create an artifact record and corresponding MinIO object, call `GET /api/hermes/artifacts/:id/url`, verify the response contains a `url` field that when fetched via HTTP GET returns 200 with `content-type: image/png`. 404 test: request a non-existent artifact ID, verify 404 response with `ARTIFACT_NOT_FOUND` error. Auth test: request without `hermes:read` permission returns 403. List test: create a deliberation with 3 artifacts, call `GET /api/hermes/deliberations/:id/artifacts`, verify 3 records are returned.