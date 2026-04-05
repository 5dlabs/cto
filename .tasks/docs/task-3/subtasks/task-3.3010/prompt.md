Implement subtask 3010: Write integration tests for all RMS gRPC and REST endpoints

## Objective
Create comprehensive integration tests covering all five services end-to-end, including the quote-to-project workflow, barcode scanning, and cross-service interactions.

## Steps
1. Set up test infrastructure: use testcontainers-go or docker-compose to spin up PostgreSQL and Redis for integration tests.
2. Write test suite for OpportunityService: create → list → update → convert to project. Verify REST endpoints via grpc-gateway return same data as gRPC.
3. Write test suite for ProjectService: create project → assign crew → assign equipment → update dates. Verify crew and equipment associations.
4. Write test suite for InventoryService: create item → scan barcode → check out → scan again → check in. Verify status transitions and barcode dedup via Redis.
5. Write test suite for CrewService: create member → assign to project → verify schedule → attempt conflicting assignment (expect failure). Verify Google Calendar mock is called.
6. Write test suite for DeliveryService: create delivery → update status through full lifecycle (SCHEDULED→IN_TRANSIT→DELIVERED→RETURNED). Verify equipment status updates on delivery.
7. Write end-to-end workflow test: Create opportunity → convert to project → assign crew → assign equipment → create delivery → complete delivery → check in equipment. This tests the full quote-to-project-to-delivery pipeline.
8. Verify all REST endpoints via HTTP client match gRPC responses.
9. Ensure test coverage report shows ≥80% across all service packages.

## Validation
All integration test suites pass in CI with containerized PostgreSQL and Redis; end-to-end workflow test completes without errors; REST and gRPC responses are consistent; coverage report shows ≥80% for /internal/service packages.