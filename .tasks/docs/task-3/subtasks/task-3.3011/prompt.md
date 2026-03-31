Implement subtask 3011: Implement DeliveryService gRPC handlers

## Objective
Implement the DeliveryService gRPC server for scheduling and tracking deliveries and pickups associated with projects.

## Steps
1. Create `/internal/service/delivery_service.go` implementing the generated DeliveryServiceServer interface.
2. Implement CreateDelivery: validate project exists, insert delivery record with SCHEDULED status.
3. Implement GetDelivery: query by ID with project info.
4. Implement ListDeliveries: support pagination, filtering by project_id, type (delivery/pickup), status, and date range.
5. Implement UpdateDeliveryStatus: validate status transitions (SCHEDULED→IN_TRANSIT→COMPLETED), update record.
6. Create `/internal/repository/delivery_repo.go`.
7. Register the service with the gRPC server.

## Validation
Unit tests for each RPC method. Integration test: create delivery for a project, transition through statuses, verify invalid transitions are rejected. List with filters returns correct subsets.