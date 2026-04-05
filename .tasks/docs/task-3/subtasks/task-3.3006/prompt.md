Implement subtask 3006: Implement Opportunity service with state machine, lead scoring, and convert-to-project

## Objective
Build the OpportunityService gRPC implementation with the full quote-to-project workflow including status state machine, lead scoring algorithm, and opportunity-to-project conversion.

## Steps
1. Create `internal/service/opportunity_svc.go` implementing the generated `OpportunityServiceServer` interface.
2. Implement the state machine in `internal/domain/opportunity_state.go`:
   - Define valid transitions: PENDING→QUALIFIED, QUALIFIED→APPROVED, APPROVED→CONVERTED. Also allow PENDING→QUALIFIED→APPROVED as composite path.
   - `ValidateTransition(current, next Status) error` returns descriptive error for invalid transitions (e.g., PENDING→CONVERTED is invalid).
   - State transitions are enforced in `UpdateOpportunity` and dedicated RPCs.
3. Implement `ScoreLead` RPC in `internal/domain/lead_scoring.go`:
   - Input: customer vetting data (bool flags: verified_identity, verified_insurance, positive_references), event_size (int), payment_history (enum: GOOD, MIXED, BAD, NEW).
   - Scoring: assign points per factor. GREEN >= 80 points, YELLOW 50-79, RED < 50. Return `LeadScore` with per-factor breakdown.
4. Implement `ApproveOpportunity` RPC: validate QUALIFIED→APPROVED transition, update status.
5. Implement `ConvertOpportunity` RPC:
   - Validate status is APPROVED.
   - Call `projectRepo.CreateFromOpportunity(ctx, orgID, oppID)` to create project with linked opportunity_id and copied line items.
   - Update opportunity status to CONVERTED.
   - Return the new Project ID.
   - Wrap in a database transaction (via pgx.Tx passed through repo).
6. Implement `CreateOpportunity`, `GetOpportunity`, `UpdateOpportunity`, `ListOpportunities` as standard CRUD delegating to repo.
7. Wire service into gRPC server registration in `cmd/server/main.go`.

## Validation
Unit tests for state machine: test all 4 states × all possible next states, verify exactly 3 valid transitions pass and all others return error. Unit tests for lead scoring with parameterized table: test GREEN boundary (80+), YELLOW (50-79), RED (<50) with various input combinations. Integration test: CreateOpportunity → ScoreLead → UpdateStatus to QUALIFIED → Approve → Convert → verify Project exists with correct opportunity_id link and line items copied.