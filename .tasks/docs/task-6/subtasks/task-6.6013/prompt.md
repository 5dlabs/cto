Implement subtask 6013: Implement approval workflow endpoints and multi-platform publishing pipeline

## Objective
Build the approve, reject, and publish endpoints with draft status state machine transitions, and the publishing pipeline that dispatches to multiple platform services concurrently with partial failure handling.

## Steps
1. Add routes to `src/routes/social.ts`:
2. `POST /api/v1/social/drafts/:id/approve`:
   - Validate draft exists and status is 'pending_approval'.
   - Transition status to 'approved'. Update `updated_at`.
   - Return updated draft.
3. `POST /api/v1/social/drafts/:id/reject`:
   - Accept optional `reason` in body.
   - Validate draft exists and status is 'pending_approval'.
   - Transition status to 'rejected'. Store reason in metadata. Update `updated_at`.
   - Return updated draft.
4. `POST /api/v1/social/drafts/:id/publish`:
   - Validate draft exists and status is 'approved'.
   - For each platform in draft.platforms, dispatch to the respective publish service concurrently using `Effect.allSettled` (or `Effect.forEach` with error collection).
   - For each successful publish: insert a `published_posts` row with platform, platform_post_id, published_at.
   - For each failed publish: log the error, do NOT insert a published_posts row.
   - After all platforms attempted: if all succeeded, set draft status to 'published'. If some failed, set status to 'published' but include failure details in response (or add a `platform_statuses` JSONB field). If all failed, set status to 'failed'.
   - Return detailed result with per-platform success/failure.
5. `GET /api/v1/social/published`:
   - Query published_posts with optional platform filter, paginated, ORDER BY published_at DESC.
   - Return with engagement_data if available.
6. Validate all state transitions — reject invalid transitions (e.g., publishing a rejected draft) with 409 Conflict.
7. Use Effect Schema for all request/response validation.

## Validation
Integration test: create a draft in 'pending_approval' status → approve → publish → verify published_posts created for each platform. Partial failure test: mock Instagram success + LinkedIn failure → verify draft reflects partial publication, published_posts exists only for Instagram. State machine test: attempt to publish a 'rejected' draft → verify 409 Conflict. Attempt to approve an already 'approved' draft → verify appropriate error.