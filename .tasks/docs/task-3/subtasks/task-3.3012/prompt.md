Implement subtask 3012: Write end-to-end integration tests for quote-to-project workflow

## Objective
Create integration tests that exercise the full quote-to-project lifecycle spanning OpportunityService, ProjectService, CrewService, InventoryService, and DeliveryService.

## Steps
1. Create `test/integration/workflow_test.go`. 2. Test Case 1 — Quote to Project: Create opportunity → update to QUOTED → ConvertToProject → verify project created with correct opportunity_id. 3. Test Case 2 — Project Staffing: Create crew members → AssignCrew to project → CheckAvailability shows them as assigned. 4. Test Case 3 — Inventory Reservation: Create inventory items → ReserveItems for a project → verify quantities updated → CreateDelivery with reserved items. 5. Test Case 4 — Delivery Lifecycle: CreateDelivery → UpdateDeliveryStatus through PENDING→IN_TRANSIT→DELIVERED → verify final state. 6. Test Case 5 — Full Workflow: Chain all above steps into a single end-to-end test measuring total time (must complete in <2 minutes per PRD). 7. Use testcontainers-go or docker-compose for PostgreSQL and Redis test dependencies. 8. Test both gRPC and REST endpoints for the workflow.

## Validation
All 5 test cases pass; full workflow completes in <2 minutes; tests run against real PostgreSQL and Redis instances (via testcontainers); both gRPC and REST paths produce identical results; test output includes timing information.