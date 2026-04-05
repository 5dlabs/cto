Implement subtask 6004: Implement draft management REST endpoints with Effect.Schema validation

## Objective
Build the CRUD endpoints for draft management: list drafts, get draft by ID, approve, reject — with full Effect.Schema request/response validation.

## Steps
1. Define Effect.Schema models for all request/response types: DraftResponse (id, image_urls, caption, ai_score, status, platform_targets, created_at), DraftListResponse (items array, total count), ApproveRequest (optional caption_override), RejectRequest (reason). 2. GET /api/v1/social/drafts: List all drafts with optional status filter query param. Return paginated DraftListResponse. 3. GET /api/v1/social/drafts/:id: Return single DraftResponse or 404. 4. POST /api/v1/social/drafts/:id/approve: Validate draft exists and is in 'draft' status. Update status to 'approved'. If caption_override provided, update caption. Return updated draft. 5. POST /api/v1/social/drafts/:id/reject: Validate draft exists and is in 'draft' status. Update status to 'rejected' with reason. Return updated draft. 6. Apply Effect.Schema validation to all endpoints using Elysia's validation hooks. 7. Return consistent error responses for validation failures (400), not found (404), and invalid state transitions (409).

## Validation
Test each endpoint: list drafts returns correct items with status filter; get by ID returns draft or 404; approve transitions status correctly; reject stores reason; invalid state transitions return 409; malformed requests return 400 with validation details.