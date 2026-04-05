Implement subtask 6010: Implement approval workflow with Signal integration

## Objective
Build the approval workflow system that sends draft content for human review via Signal messaging, handles approve/reject responses, and updates draft status accordingly.

## Steps
1. Create src/services/approval-workflow.ts.
2. Define Effect.Service `ApprovalWorkflowService` with methods: requestApproval(draft: Draft) -> Effect<void>, handleApproval(draftId: UUID) -> Effect<Draft>, handleRejection(draftId: UUID, reason: string) -> Effect<Draft>.
3. Implement Signal notification: send a message via Signal REST API (or signal-cli HTTP API) containing draft preview (image thumbnail URLs, caption text, target platforms) and approve/reject action links.
4. Message format: include draft ID, preview of caption, platform targets, and instructions.
5. Generate unique approval/rejection URLs that map to the draft endpoints.
6. Update draft status in database on approval or rejection.
7. Handle Signal API connectivity issues with retry logic.
8. Log all approval actions for audit trail.
9. Implement mock for testing.

## Validation
Unit tests verify Signal API is called with correct message format containing draft details; approval updates draft status to Approved; rejection updates to Rejected with reason; retry logic handles Signal API failures; audit log entries are created.