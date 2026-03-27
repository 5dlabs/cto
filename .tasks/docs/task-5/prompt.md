Implement task 5: RMS Service - Inventory, Crew, Delivery gRPC & REST (Grizz - Go/gRPC)

## Goal
Expand the RMS service to include Inventory, Crew, and Delivery management. This involves defining new protobuf services, implementing their logic, and extending the REST gateway.

## Task Context
- Agent owner: Grizz
- Stack: Go/gRPC
- Priority: medium
- Dependencies: 1, 4

## Implementation Plan
1. Define protobuf schemas for `InventoryService`, `CrewService`, and `DeliveryService`, including RPCs like `GetStockLevel`, `RecordTransaction`, `ScanBarcode`, `ListCrew`, `AssignCrew`, `ScheduleCrew`, `ScheduleDelivery`, `UpdateDeliveryStatus`, `OptimizeRoute`. 2. Generate Go code from new protobuf definitions. 3. Implement the gRPC server for these new services. 4. Extend the PostgreSQL schema to support `InventoryTransaction`, `CrewMember`, `Delivery` models. 5. Implement the logic for each RPC, including database interactions for inventory transactions, crew assignments, and delivery scheduling. 6. Extend `grpc-gateway` to expose REST endpoints for these services (e.g., `GET /api/v1/inventory/transactions`, `POST /api/v1/inventory/transactions`, `GET /api/v1/crew`, `POST /api/v1/crew/assign`, `POST /api/v1/deliveries/schedule`). 7. Implement a stub for Google Calendar API integration for crew scheduling, logging calls without actual external interaction for now. Use Go 1.22+.

## Acceptance Criteria
1. Use `grpcurl` and `curl` to test all new gRPC and REST endpoints for Inventory, Crew, and Delivery services. 2. Verify that inventory transactions are correctly recorded and stock levels are updated. 3. Confirm crew members can be listed and assigned to projects. 4. Verify delivery schedules can be created and updated. 5. Check that the Google Calendar API stub logs expected calls. 6. Run `staticcheck ./...` to identify potential issues in the Go codebase. 7. Ensure all new database models are correctly created and data integrity is maintained.

## Subtasks
- Implement RMS Service - Inventory, Crew, Delivery gRPC & REST (Grizz - Go/gRPC): Expand the RMS service to include Inventory, Crew, and Delivery management. This involves defining new protobuf services, implementing their logic, and extending the REST gateway.

## Deliverables
- Update the relevant code, configuration, and tests.
- Keep artifacts aligned with the acceptance criteria.
- Document blockers or assumptions in your final summary.