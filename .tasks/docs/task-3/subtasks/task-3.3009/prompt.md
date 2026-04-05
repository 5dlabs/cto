Implement subtask 3009: Implement DeliveryService with tracking

## Objective
Implement gRPC handlers for DeliveryService including delivery CRUD, status tracking, and delivery state machine transitions.

## Steps
1. Implement DeliveryService handlers in /internal/delivery/service.go: CreateDelivery (link to project, set initial status to 'pending', validate pickup/delivery addresses). 2. GetDelivery, ListDeliveries (filter by project_id, status, date range). 3. Implement UpdateDeliveryStatus RPC with state machine validation: pending→in_transit→delivered, or pending→in_transit→returned. Invalid transitions return FailedPrecondition. When status changes to 'delivered', set actual_date to now. 4. Implement TrackDelivery RPC: return current status, scheduled vs actual dates, and driver notes for a given delivery. 5. Use proper gRPC error codes for invalid transitions and not-found cases.

## Validation
Unit tests verify valid and invalid state transitions; CreateDelivery links to project; TrackDelivery returns correct current state; integration test runs full pending→in_transit→delivered flow; >80% coverage.