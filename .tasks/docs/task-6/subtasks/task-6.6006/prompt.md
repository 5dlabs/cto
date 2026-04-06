Implement subtask 6006: Implement approval workflow with Signal notification to Morgan

## Objective
Build the approve/reject endpoints and integrate Signal messaging to notify Morgan when drafts need review and when approval decisions are made.

## Steps
1. In `src/routes/drafts.ts`, add: a) `POST /api/v1/social/drafts/:id/approve` — validates draft exists and is PENDING_REVIEW, updates status to APPROVED, stores reviewer_notes, returns 200. b) `POST /api/v1/social/drafts/:id/reject` — validates draft exists and is PENDING_REVIEW, updates status to REJECTED, stores rejection reason, returns 200. 2. In `src/services/signal-notify.ts`, create an Effect.Service `SignalNotificationService` with: a) `notifyDraftReady(draft: Draft) -> Effect<void>` — sends Signal message to Morgan with draft preview (caption text, image count, target platforms) and approval link. b) `notifyApprovalDecision(draft: Draft, decision: 'approved' | 'rejected') -> Effect<void>` — confirms decision. 3. Trigger `notifyDraftReady` when a new draft is created (from 6005). 4. Trigger `notifyApprovalDecision` after approve/reject. 5. Validate state transitions (can only approve/reject PENDING_REVIEW drafts). 6. Handle Signal API failures gracefully (log and continue, don't block approval flow).

## Validation
POST /approve on PENDING_REVIEW draft changes status to APPROVED; POST /reject changes to REJECTED; attempting to approve an already-approved draft returns 409; Signal notification is sent on draft creation and approval (verified via mock); Signal failure does not block the approval flow.