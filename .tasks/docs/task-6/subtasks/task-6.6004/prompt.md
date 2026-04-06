Implement subtask 6004: Implement draft management endpoints and approval workflow

## Objective
Build the draft CRUD endpoints and the approval/rejection workflow including Signal notification integration with Morgan.

## Steps
1. Implement GET `/api/v1/social/drafts` — list all drafts with filtering by status, pagination.
2. Implement GET `/api/v1/social/drafts/:id` — get single draft with full details.
3. Implement POST endpoint or internal function to create a draft:
   - Accept image_urls (from upload), run AI curation to select best images, generate captions per target platform.
   - Store draft in PostgreSQL with status 'pending_review'.
4. Implement POST `/api/v1/social/drafts/:id/approve`:
   - Validate draft exists and is in 'pending_review' status.
   - Update status to 'approved'.
5. Implement POST `/api/v1/social/drafts/:id/reject`:
   - Accept optional rejection reason.
   - Update status to 'rejected'.
6. Implement Signal/Morgan notification integration:
   - Create `src/notifications/signal.ts` module.
   - When a draft is created, send a Signal message to the configured approval contact with draft preview and approve/reject links.
   - Read SIGNAL_API_ENDPOINT and MORGAN_WEBHOOK_URL from environment.
7. Validate all request/response schemas with Effect.Schema.
8. Add proper state machine validation (can't approve an already published draft, etc.).

## Validation
Integration tests: create a draft triggers AI curation and persists with 'pending_review' status; GET /drafts returns paginated list; approve transitions to 'approved'; reject transitions to 'rejected'; invalid state transitions return 409; Signal notification is sent on draft creation (verified via mock).