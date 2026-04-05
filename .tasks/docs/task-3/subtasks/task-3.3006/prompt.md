Implement subtask 3006: Implement OpportunityService gRPC handlers with ScoreLead logic

## Objective
Implement the OpportunityService gRPC server with full CRUD operations for opportunities and line items, plus the ScoreLead RPC that computes GREEN/YELLOW/RED based on vetting data and opportunity value.

## Steps
1. Create `internal/service/opportunity.go` implementing the generated OpportunityServiceServer interface.
2. Implement CreateOpportunity: validate required fields (customer_id, event dates), insert opportunity, return created resource.
3. Implement GetOpportunity: lookup by ID, include nested line items, return NOT_FOUND if missing.
4. Implement UpdateOpportunity: support partial updates via field mask or by checking non-zero values, enforce valid status transitions (pending→qualified→approved→converted; no skipping, no backward).
5. Implement ListOpportunities: support pagination (page_size default 20, max 100, page_token as opaque cursor), optional status filter.
6. Implement ScoreLead:
   - GREEN: customer has verified vetting data AND opportunity total > $5000
   - YELLOW: customer has partial vetting data OR opportunity total between $1000-$5000
   - RED: customer has no vetting data AND opportunity total > $5000 (high risk)
   - Store computed lead_score on the opportunity record
   - Return the score in the response
7. Implement CreateOpportunity with line items: accept repeated line items in the create request, compute subtotal_cents = quantity * day_rate_cents * days, compute total_estimate_cents as sum of all line item subtotals.
8. Input validation: use `google.golang.org/grpc/codes` and `status` for proper gRPC error codes (InvalidArgument, NotFound, FailedPrecondition for invalid state transitions).
9. Register the service in the gRPC server in main.go.

## Validation
Unit test ScoreLead: GREEN for verified+>$5000, YELLOW for partial vetting, RED for no vetting+high value (minimum 3 scenarios as specified). Unit test status transition validation: verify approved→converted succeeds, pending→converted fails. Integration test: CreateOpportunity with line items and verify total_estimate_cents is computed correctly.