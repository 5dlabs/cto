Implement subtask 3007: Implement DeliveryService with logistics tracking

## Objective
Build the DeliveryService for scheduling equipment deliveries and pickups linked to projects, with status tracking through the delivery lifecycle.

## Steps
1. Create `internal/delivery/` package with repository, service, and handler layers.
2. Implement `repository.go`: CreateDelivery, GetDeliveryByID, ListDeliveries (filter by project, status, date range), UpdateDeliveryStatus.
3. Define delivery status flow: scheduled → in_transit → delivered (for deliveries) and scheduled → in_transit → picked_up (for pickups). Validate transitions.
4. Implement ScheduleDelivery: create a delivery or pickup record linked to a project_id, with address, scheduled_at, and optional notes.
5. Implement UpdateDeliveryStatus: validate status transition, record timestamp of each status change.
6. Implement ListDeliveries with filtering by project_id, type (delivery/pickup), status, and date range.
7. Wire up gRPC handlers.

## Validation
Integration tests: schedule a delivery for a project, transition through statuses, verify each transition is recorded with timestamps. Verify invalid transitions are rejected. List deliveries filtered by project and status returns correct results. REST endpoints via grpc-gateway return proper JSON.