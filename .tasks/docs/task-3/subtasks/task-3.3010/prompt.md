Implement subtask 3010: Implement Delivery management service

## Objective
Build the DeliveryService gRPC implementation with delivery scheduling, status updates, route optimization, and listing.

## Steps
1. Create `internal/service/delivery_svc.go` implementing `DeliveryServiceServer`.
2. Implement `ScheduleDelivery` RPC:
   - Accept project_id, type (DELIVERY/PICKUP), address, scheduled_at.
   - Validate project exists via projectRepo.
   - Create delivery record via deliveryRepo.Create().
   - Return created Delivery with ID.
3. Implement `UpdateDeliveryStatus` RPC:
   - Accept delivery_id and new status.
   - Validate status transition: SCHEDULED→IN_TRANSIT→DELIVERED, or SCHEDULED→CANCELLED.
   - Update via deliveryRepo.UpdateStatus().
4. Implement `OptimizeRoute` RPC:
   - Accept list of delivery_ids.
   - Fetch all deliveries with addresses.
   - Implement a simple nearest-neighbor heuristic for route ordering (v1 — no external API dependency).
   - Save DeliveryRoute with optimized_order and estimated_duration.
   - Return DeliveryRoute.
5. Implement `ListDeliveries` with org_id filtering, optional project_id filter, and pagination.
6. Register service in gRPC server.

## Validation
Integration tests: 1) ScheduleDelivery for existing project → verify delivery created with SCHEDULED status. 2) UpdateDeliveryStatus SCHEDULED→IN_TRANSIT→DELIVERED → verify each transition. 3) Attempt invalid transition DELIVERED→SCHEDULED → verify error. 4) OptimizeRoute with 3 deliveries → verify route returned with all delivery_ids, optimized_order has length 3. 5) ListDeliveries filtered by project_id returns only related deliveries.