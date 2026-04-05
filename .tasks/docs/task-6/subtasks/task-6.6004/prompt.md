Implement subtask 6004: Implement upload endpoint and draft listing/detail endpoints

## Objective
Build the Elysia routes for multipart image upload (POST /api/v1/social/upload), draft listing with pagination and status filter (GET /api/v1/social/drafts), and draft detail (GET /api/v1/social/drafts/:id). Integrate R2 for file storage and database for record persistence.

## Steps
1. Create `src/routes/social.ts` as an Elysia plugin/group under `/api/v1/social`.
2. `POST /api/v1/social/upload`:
   - Accept multipart/form-data with multiple image files and optional `event_id`.
   - For each file: validate it's an image (MIME check), generate a UUID-based R2 key, upload via R2StorageService, create an `uploads` row with original_url = R2 key, metadata (extract dimensions if possible).
   - Return 201 with array of created upload records.
   - After response, trigger the AI curation pipeline asynchronously (emit an event or call a pipeline function — wire actual AI logic in subtask 6005/6006/6007).
3. `GET /api/v1/social/drafts`:
   - Query params: `status` (optional filter), `page` (default 1), `limit` (default 20).
   - Query drafts table with optional status filter, ORDER BY created_at DESC, LIMIT/OFFSET pagination.
   - Return paginated response: `{ data: Draft[], total: number, page: number, limit: number }`.
4. `GET /api/v1/social/drafts/:id`:
   - Fetch draft by UUID, include presigned URLs for each upload_id image and each platform crop.
   - Return 404 if not found.
5. Define Effect Schema validators for request bodies and query params.
6. Use Effect.gen for route handler composition, providing all required service layers.

## Validation
Integration test: POST /api/v1/social/upload with 3 test images → verify 201 response with 3 upload records, each having a valid R2 key. GET /api/v1/social/drafts returns paginated list. GET /api/v1/social/drafts/:id returns draft with image URLs. POST with invalid MIME type returns 422. GET non-existent draft ID returns 404.