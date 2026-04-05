Implement subtask 3005: Implement InventoryService with barcode scanning logic

## Objective
Build the InventoryService gRPC handler with barcode scanning, check-out/check-in workflows, and inventory status management.

## Steps
1. Create /internal/service/inventory_service.go implementing InventoryServiceServer.
2. Implement CreateItem: validate barcode uniqueness, persist new inventory item with AVAILABLE status.
3. Implement GetItem and ListItems: support filtering by category, status, location; pagination.
4. Implement ScanBarcode: accept a barcode string, look up the item, update last_scanned_at timestamp, return item details and current status. This is the core barcode scanning endpoint.
5. Implement UpdateItemStatus: transition item status with validation (e.g., can't go from MAINTENANCE directly to CHECKED_OUT).
6. Implement CheckOut: accept item_id and project_id, validate item is AVAILABLE or RESERVED, update status to CHECKED_OUT, record association with project via project_equipment.
7. Implement CheckIn: accept item_id, validate item is CHECKED_OUT, update status back to AVAILABLE, remove project association, update last_scanned_at.
8. Add status transition validation as a state machine helper to enforce valid inventory state transitions.
9. Register service on gRPC server and grpc-gateway mux.

## Validation
ScanBarcode returns correct item for a known barcode and 404 for unknown; CheckOut transitions item to CHECKED_OUT and fails if item is in MAINTENANCE; CheckIn returns item to AVAILABLE; status transition state machine rejects invalid transitions; barcode uniqueness constraint is enforced on creation.