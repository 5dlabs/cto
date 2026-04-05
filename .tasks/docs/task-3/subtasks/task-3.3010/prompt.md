Implement subtask 3010: Implement DeliveryService gRPC handlers

## Objective
Implement the DeliveryService gRPC server with delivery scheduling, status updates, and a stub OptimizeRoute endpoint for v1.

## Steps
1. Create `internal/service/delivery.go` implementing DeliveryServiceServer.
2. Implement ScheduleDelivery:
   - Accept project_id, scheduled_at, vehicle_id, driver_id, pickup_address, delivery_address, notes
   - Validate project exists
   - Create delivery record with status=SCHEDULED
   - Return created delivery
3. Implement UpdateDeliveryStatus:
   - Accept delivery_id and new status
   - Validate status transitions: SCHEDULED→IN_TRANSIT→DELIVERED, SCHEDULED→CANCELLED, IN_TRANSIT→CANCELLED
   - Update record, return updated delivery
4. Implement OptimizeRoute:
   - v1 stub: accept list of delivery_ids, return them in the same order
   - Include a response field `optimized: false` and `message: 'Route optimization not yet implemented'`
   - This keeps the API contract stable for future implementation
5. Register service in gRPC server.

## Validation
Integration test: create delivery, verify status is SCHEDULED. Update to IN_TRANSIT, then DELIVERED — verify each transition. Test invalid transition (DELIVERED→IN_TRANSIT) returns error. Test OptimizeRoute returns input deliveries with optimized=false.