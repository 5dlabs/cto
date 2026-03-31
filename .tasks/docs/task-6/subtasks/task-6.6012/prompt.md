Implement subtask 6012: Add Effect.Schema validation for all request/response contracts

## Objective
Define and enforce Effect.Schema validators on every Elysia route for request bodies, query parameters, path parameters, and response shapes across all social media endpoints.

## Steps
1. Create `src/schemas/social.ts` consolidating all schemas: PhotoUploadRequest, PhotoUploadResponse, DraftCreateRequest, DraftCreateResponse, DraftListQuery (status, platform, limit, offset), DraftResponse, ApproveRequest, RejectRequest (with optional rejection_reason), PublishResponse, PublishedListQuery, PublishedResponse. 2. Use Effect.Schema decorators/annotations for OpenAPI metadata (description, examples). 3. Wire schemas into Elysia route definitions using `.body()`, `.query()`, `.params()`, `.response()` type guards provided by @elysiajs/swagger or a custom Effect.Schema-to-Elysia adapter. 4. Implement a global error handler that catches Effect.Schema DecodeError and returns standardized 400 responses with field-level error details. 5. Ensure all response payloads are encoded through Effect.Schema to strip internal fields (e.g., encrypted tokens). 6. Generate OpenAPI/Swagger documentation from the schemas at /api/v1/social/docs.

## Validation
Send malformed JSON to each endpoint and verify 400 response with field-level errors. Send valid requests and verify response shapes match schema exactly. Access /api/v1/social/docs and verify OpenAPI spec lists all endpoints with correct schemas. Verify internal fields are not leaked in responses.