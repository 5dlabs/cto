Implement subtask 6006: Implement approval workflow with Signal integration via Morgan

## Objective
Build the approval workflow that sends draft content for review via Signal (through Morgan integration) and processes approval/rejection decisions.

## Steps
1. Create `src/services/approval.ts` module using Effect.Service pattern.
2. Define an `ApprovalService` with methods: `requestApproval(draftId: string) -> Effect.Effect<ApprovalRequest, ApprovalError>`, `processDecision(approvalId: string, decision: 'approved' | 'rejected', approvedBy: string) -> Effect.Effect<ApprovalResult, ApprovalError>`.
3. Implement approval request flow:
   - Fetch the draft with photos and caption.
   - Send a message to Morgan's API (or webhook) containing draft preview (image URL, caption text, platform).
   - Morgan forwards to Signal for human review.
   - Update draft status to 'pending_approval'.
   - Store the approval request in the `approvals` table.
4. Implement approval callback:
   - POST `/api/v1/social/approvals/:id/decide` — accept decision (approved/rejected) with optional feedback.
   - Update the `approvals` table with decision and timestamp.
   - Update the `drafts` table status accordingly.
   - If approved, optionally auto-trigger publishing.
5. Implement GET `/api/v1/social/approvals` — list pending approvals.
6. Implement GET `/api/v1/social/approvals/:id` — get approval details.
7. Handle Morgan unavailability gracefully — queue the approval request for retry.

## Validation
Mock Morgan API. Verify requesting approval sends the correct payload to Morgan and updates draft status. Verify approval decision updates both approvals and drafts tables. Verify rejection flow works. Test Morgan unavailability triggers retry logic. List pending approvals and verify filtering.