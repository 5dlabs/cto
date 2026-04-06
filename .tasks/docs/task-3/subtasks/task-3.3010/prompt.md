Implement subtask 3010: Write integration tests for all RMS gRPC and REST endpoints

## Objective
Create comprehensive integration tests covering all 5 services via both gRPC and REST interfaces, targeting >80% code coverage.

## Steps
1. Set up a test harness using testcontainers-go or docker-compose to spin up PostgreSQL and Redis for integration tests.
2. Create test fixtures/factories for opportunities, projects, inventory items, crew members, and deliveries.
3. Write gRPC client tests for each service:
   - OpportunityService: full CRUD lifecycle, list with pagination, invalid input handling
   - ProjectService: CRUD lifecycle, calendar event creation (mocked), opportunity linkage validation
   - InventoryService: CRUD, barcode lookup, check-in/check-out lifecycle, availability query, audit log verification
   - CrewService: CRUD, assignment with conflict detection, availability check with skill filtering
   - DeliveryService: CRUD, status transitions (valid and invalid), driver assignment conflicts
4. Write HTTP client tests via grpc-gateway for a representative subset of endpoints to verify REST mapping.
5. Test error scenarios: not found, duplicate creation, invalid state transitions, conflict detection.
6. Add a CI-compatible test script that runs all integration tests and reports coverage.
7. Verify coverage meets >80% threshold using go test -coverprofile.

## Validation
All integration tests pass in CI; gRPC and REST endpoints return expected status codes and payloads; conflict detection tests verify double-booking is prevented; barcode scanning tests verify check-in/check-out workflow; coverage report shows >80% line coverage across all service packages.