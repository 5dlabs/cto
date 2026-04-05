Implement subtask 3011: Write integration tests for end-to-end RMS workflows

## Objective
Create integration tests that exercise the full quote-to-project workflow, barcode scanning, crew scheduling, and delivery tracking across all five services using a real database.

## Steps
1. Set up test infrastructure using testcontainers-go to spin up PostgreSQL and Redis containers. 2. Run migrations before tests. 3. Test 1 - Quote-to-Project E2E: Create opportunity → Update to quoted → Update to accepted → ConvertToProject → Verify project exists with correct data. 4. Test 2 - Inventory Lifecycle: Create inventory items → ScanBarcode to verify → CheckoutItems to a project → ScanBarcode again to see 'rented' status → ReturnItems → Verify status back to 'available'. 5. Test 3 - Crew Scheduling: Create crew member → ScheduleCrewMember to project A → Attempt double-book to project B in same dates (expect failure) → Schedule to non-overlapping dates (expect success). 6. Test 4 - Delivery Tracking: Create delivery for project → UpdateStatus to in_transit → UpdateStatus to delivered → TrackDelivery returns correct state and actual_date. 7. Test 5 - REST Gateway: Repeat key flows via HTTP REST endpoints to verify grpc-gateway mapping. 8. All tests clean up after themselves or use separate schemas/transactions.

## Validation
All 5 integration test scenarios pass; tests run in CI with testcontainers; no test pollution between scenarios; REST and gRPC endpoints produce equivalent results for the same operations.