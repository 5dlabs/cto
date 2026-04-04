Implement subtask 6004: Implement draft management REST endpoints with Effect Schema validation

## Objective
Build the draft listing, detail, approve, reject, and publish endpoints. Also implement the published posts listing endpoint. All endpoints use Effect Schema for request/response validation.

## Steps
1. Create `src/routes/drafts.ts` with Elysia route group.
2. `GET /api/v1/social/drafts`:
   - Query params: `status` (optional, one of draft/approved/rejected/published/failed), `page` (int, default 1), `limit` (int, default 20, max 100).
   - Effect Schema validation on query params.
   - SQL query with optional WHERE status filter, ORDER BY created_at DESC, LIMIT/OFFSET pagination.
   - Return { drafts: [...], total: number, page, limit }.
3. `GET /api/v1/social/drafts/:id`:
   - Fetch draft by UUID with joined photo data (via upload_id → photos).
   - Return draft with nested photos array including r2_key → CDN URL via R2Service.getPublicUrl.
   - 404 if not found.
4. `POST /api/v1/social/drafts/:id/approve`:
   - Body: { approved_by: string }.
   - Validate draft exists and status is 'draft'. Return 409 if already approved/rejected/published.
   - Update status to 'approved', set approved_by, approved_at.
   - Return updated draft.
   - Note: NATS publish will be wired in subtask 6008.
5. `POST /api/v1/social/drafts/:id/reject`:
   - Body: { rejected_by: string, reason?: string }.
   - Validate draft exists and status is 'draft'. Return 409 if not in draft status.
   - Update status to 'rejected'.
   - Return updated draft.
6. `POST /api/v1/social/drafts/:id/publish`:
   - Manual publish trigger. Validate draft status is 'approved'.
   - Return 202 accepted (actual publishing handled asynchronously via NATS, wired in 6008).
7. `GET /api/v1/social/published`:
   - Query params: platform (optional), page, limit.
   - Join published_posts with drafts for context.
   - Return { posts: [...], total, page, limit }.
8. Create Effect Schema definitions in `src/schemas/social.ts` for all request/response types.
9. Use Elysia's `.onError` hook for standardized error responses with proper HTTP status codes.

## Validation
Test each endpoint: (1) GET /drafts with status filter returns only matching drafts. (2) GET /drafts/:id returns 404 for nonexistent UUID, 200 with photos for valid draft. (3) POST /approve on draft status returns 200 with updated status, POST /approve on already-approved returns 409. (4) POST /reject with reason stores reason. (5) POST /publish on non-approved draft returns 409. (6) GET /published with platform filter. (7) Schema validation: send malformed body to POST /approve, verify 422 response.