Implement subtask 3007: Implement DeliveryService with delivery tracking logic

## Objective
Build the DeliveryService gRPC handler for scheduling, tracking, and managing equipment deliveries tied to projects.

## Steps
1. Create /internal/service/delivery_service.go implementing DeliveryServiceServer.
2. Implement CreateDelivery: accept project_id, equipment_ids, driver_id, scheduled times; validate project exists and equipment items are in RESERVED or CHECKED_OUT status; persist delivery with SCHEDULED status; link equipment via delivery_equipment join table.
3. Implement GetDelivery and ListDeliveries: support filtering by project_id, status, driver_id, date range; pagination.
4. Implement UpdateDeliveryStatus: enforce valid status transitions (SCHEDULED→IN_TRANSIT→DELIVERED, DELIVERED→RETURNED). On transition to DELIVERED, update linked equipment items' location. On RETURNED, trigger check-in logic for equipment.
5. Implement TrackDelivery: return current delivery status, driver info, tracking notes, and ETA based on scheduled times.
6. Add tracking notes append functionality: allow adding timestamped notes to a delivery record for driver updates.
7. Register service on gRPC server and grpc-gateway mux.

## Validation
CreateDelivery fails if project doesn't exist or equipment is unavailable; UpdateDeliveryStatus enforces valid transitions and rejects invalid ones (e.g., SCHEDULED→RETURNED); DELIVERED transition updates equipment location; TrackDelivery returns current status and notes.