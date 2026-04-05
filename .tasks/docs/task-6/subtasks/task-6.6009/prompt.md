Implement subtask 6009: Build publishing orchestrator and published posts endpoint

## Objective
Implement the publishing orchestrator that dispatches approved drafts to all target platforms concurrently, handles partial failures, and exposes the publish and published-posts endpoints.

## Steps
1. Create a `services/publish-orchestrator.ts` module. 2. Implement `publishDraft(draftId: string): Effect.Effect<PublishSummary, PublishError>` that: a) Validates draft status is 'approved'. b) Resolves target platforms from draft.platform_targets. c) Dispatches to all target platform publishers concurrently using Effect.all with { concurrency: 'unbounded' }. d) Collects results: track per-platform success/failure. e) If all succeed: transition draft to 'published'. If partial: transition to 'published' with warnings stored. If all fail: keep as 'approved' with error details. 3. Wire up POST /api/v1/social/drafts/:id/publish to trigger the orchestrator. 4. Wire up GET /api/v1/social/published to list all published posts, filterable by platform and date range. 5. Return a PublishSummary with per-platform status, post IDs, and any errors. 6. Add Effect.Schema validation on all endpoints.

## Validation
Publishing an approved draft targeting all three platforms dispatches concurrently and returns a summary. Partial failure (e.g., Instagram succeeds, LinkedIn fails) still marks the draft with appropriate status and stores error details. GET /published returns published posts filterable by platform. Invalid publish attempts (non-approved drafts) return 409. Effect.Schema validates all request/response shapes.