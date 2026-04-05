Implement subtask 6005: Implement approval workflow with Signal/Morgan integration

## Objective
Build the approval workflow that sends draft posts for human review via Signal (through Morgan integration), handles approve/reject responses, and manages the draft lifecycle endpoints.

## Steps
1. Create a `services/approval.ts` module. 2. Implement the notification flow (pending dp-16): send an approval request to Morgan/Signal with draft summary (caption preview, photo thumbnails, target platforms). Include approve/reject action links or commands. 3. Implement the draft management endpoints: a) GET /api/v1/social/drafts — list all drafts, filterable by status. b) GET /api/v1/social/drafts/:id — get single draft with full details. c) POST /api/v1/social/drafts/:id/approve — transition status to 'approved', record approver. d) POST /api/v1/social/drafts/:id/reject — transition status to 'rejected', accept optional rejection_reason. 4. Validate state transitions: only 'pending_approval' drafts can be approved/rejected. Return 409 Conflict for invalid transitions. 5. On approval, emit an event or flag the draft as ready for publishing. 6. Add Effect.Schema validation on all draft endpoints for request parameters and bodies.

## Validation
Creating a draft in 'pending_approval' status triggers a notification to Morgan/Signal. Approve endpoint transitions draft to 'approved'. Reject endpoint transitions to 'rejected' with reason. Invalid state transitions (e.g., approving an already published draft) return 409. GET /drafts returns filtered lists. GET /drafts/:id returns full draft details. Effect.Schema rejects invalid request bodies.