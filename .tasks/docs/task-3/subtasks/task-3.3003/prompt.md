Implement subtask 3003: Define protobuf schemas for InventoryService

## Objective
Author .proto files for InventoryService covering equipment items, availability tracking, barcode associations, and check-in/check-out operations.

## Steps
1. Create proto/rms/v1/inventory.proto with:
   - InventoryItem message (id, name, category, barcode, serial_number, status, location, condition, last_checked_at)
   - CheckInRequest/CheckOutRequest messages (item_id or barcode, project_id, crew_member_id, timestamp, notes)
   - AvailabilityQuery message (date_range, category, location)
   - RPCs: CreateItem, GetItem, ListItems, UpdateItem, CheckIn, CheckOut, QueryAvailability, GetItemByBarcode
   - google.api.http annotations for all RPCs
2. Run code generation and verify compilation.
3. Register generated service in grpc-gateway mux.

## Validation
Generated Go code compiles; all RPC signatures and message fields match PRD inventory requirements; barcode-based lookup RPC is present; HTTP route annotations are correct.