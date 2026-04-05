Implement subtask 6016: Define Effect Schema validators for all request/response types

## Objective
Create comprehensive Effect Schema definitions for all API request bodies, query parameters, and response types to ensure type-safe validation across all endpoints.

## Steps
1. Create `src/schemas/` directory with files per domain:
   - `src/schemas/upload.ts`: UploadRequest (multipart schema), UploadResponse (array of upload records).
   - `src/schemas/draft.ts`: DraftListQuery (status, page, limit), DraftResponse, DraftDetailResponse (with presigned URLs and crops), ApproveRequest (empty body or optional metadata), RejectRequest ({ reason?: string }), PublishRequest (optional platform override).
   - `src/schemas/published.ts`: PublishedListQuery (platform, page, limit), PublishedPostResponse (with engagement_data).
   - `src/schemas/common.ts`: PaginatedResponse<T>, ErrorResponse, UUIDParam.
2. Use `@effect/schema` (Schema.Struct, Schema.String, Schema.Number, etc.).
3. Add proper constraints: UUID format validation, status enum validation, page/limit bounds (min 1, max 100), platform enum validation.
4. Wire schemas into Elysia routes using `t.` type definitions or custom validation hooks that call `Schema.decodeUnknownEither`.
5. Ensure 422 responses are structured with field-level error details.

## Validation
Test each schema with valid input → verify successful decode. Test with missing required fields → verify 422 with structured error naming the missing field. Test with invalid UUID format → verify rejection. Test page/limit bounds — page=0 or limit=200 rejected.