Implement subtask 3003: Define inventory.proto, crew.proto, and delivery.proto with grpc-gateway annotations

## Objective
Create protobuf definitions for the Inventory, Crew, and Delivery gRPC services with full REST gateway annotations.

## Steps
1. Create `proto/rms/v1/inventory.proto`:
   - Service `InventoryService` with RPCs: `GetStockLevel`, `RecordTransaction`, `ScanBarcode`, `ListInventoryItems`.
   - Messages: `InventoryItem` (id, org_id, name, barcode, category, current_location, status enum [AVAILABLE, CHECKED_OUT, MAINTENANCE, RETIRED], quantity_total, quantity_available), `InventoryTransaction` (id, item_id, project_id, type enum [CHECK_OUT, CHECK_IN, TRANSFER, ADJUSTMENT], quantity, timestamp), `ScanBarcodeResponse` (item with current location and status).
   - grpc-gateway: `GET /api/v1/inventory/{id}/stock`, `POST /api/v1/inventory/transactions`, `POST /api/v1/inventory/scan`, `GET /api/v1/inventory`.
2. Create `proto/rms/v1/crew.proto`:
   - Service `CrewService` with RPCs: `ListCrew`, `AssignCrew`, `ScheduleCrew`, `GetCrewAvailability`.
   - Messages: `CrewMember` (id, org_id, name, email, role, skills), `CrewAssignment` (id, crew_member_id, project_id, date_start, date_end, role), `AvailabilitySlot`.
   - grpc-gateway: `GET /api/v1/crew`, `POST /api/v1/crew/assign`, `POST /api/v1/crew/schedule`, `GET /api/v1/crew/{id}/availability`.
3. Create `proto/rms/v1/delivery.proto`:
   - Service `DeliveryService` with RPCs: `ScheduleDelivery`, `UpdateDeliveryStatus`, `OptimizeRoute`, `ListDeliveries`.
   - Messages: `Delivery` (id, org_id, project_id, type enum [DELIVERY, PICKUP], address, scheduled_at, status enum [SCHEDULED, IN_TRANSIT, DELIVERED, CANCELLED]), `DeliveryRoute` (id, delivery_ids, optimized_order, estimated_duration).
   - grpc-gateway: `POST /api/v1/deliveries`, `PUT /api/v1/deliveries/{id}/status`, `POST /api/v1/deliveries/optimize-route`, `GET /api/v1/deliveries`.
4. Run `buf generate` and verify all stubs compile.

## Validation
Run `buf lint` with zero errors on all three proto files. Run `buf generate` and verify all Go service interfaces and message types are generated. Confirm grpc-gateway annotations produce correct HTTP paths.