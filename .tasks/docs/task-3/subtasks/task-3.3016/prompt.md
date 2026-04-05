Implement subtask 3016: Write end-to-end integration tests for full RMS service lifecycle

## Objective
Create comprehensive integration tests covering the full opportunity-to-project lifecycle, crew conflict detection, inventory tracking, and both gRPC and REST interfaces.

## Steps
1. Create `tests/integration/` directory with test setup:
   - Use testcontainers-go to spin up PostgreSQL and Valkey containers
   - Run migrations against test DB
   - Start gRPC and HTTP servers on random available ports
2. Test full lifecycle:
   - CreateOpportunity with line items → ScoreLead → UpdateOpportunity(status=qualified) → UpdateOpportunity(status=approved) → CreateProject → verify opportunity is now 'converted'
   - CheckOut inventory from project → verify stock level decreased → CheckIn → verify stock level restored
3. Test crew scheduling conflicts:
   - AssignCrew member to project 1 (10am-2pm) → success
   - AssignCrew same member to project 2 (1pm-5pm) → expect AlreadyExists error with conflict details
   - AssignCrew same member to project 3 (3pm-6pm) → success (no overlap with 10am-2pm)
4. Test gRPC reflection: connect with grpcurl, `list` returns all 5 service names.
5. Test REST via grpc-gateway:
   - GET /api/v1/opportunities returns JSON array
   - POST /api/v1/opportunities creates resource and returns JSON with snake_case fields
   - Verify Authorization header is required (returns 401 without it)
6. Test barcode scan: insert test barcode mapping, ScanBarcode returns correct item.
7. Test health endpoints: /health/live returns 200, /health/ready returns 200 with healthy DB.
8. Test Prometheus metrics: make several RPCs, verify /metrics contains grpc_server_handled_total.

## Validation
All integration tests pass in CI. Full lifecycle test creates and verifies at least 5 database records across multiple tables. Crew conflict test validates both positive and negative cases. REST tests verify JSON response format matches PRD. gRPC reflection returns exactly 5 service names. Metrics endpoint returns expected counters.