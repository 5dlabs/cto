Implement subtask 3007: Implement InventoryService gRPC handlers with barcode scanning logic

## Objective
Implement InventoryService gRPC handlers including barcode-based lookup, check-in/check-out workflows that update item status and create audit records.

## Steps
1. Create internal/service/inventory_service.go implementing InventoryServiceServer.
2. Implement CRUD RPCs (CreateItem, GetItem, ListItems, UpdateItem) wired to the repository.
3. Implement GetItemByBarcode RPC that looks up items by barcode string.
4. Implement CheckOut RPC:
   - Validate item exists and is in 'available' status
   - Validate project_id exists
   - Update item status to 'checked_out' with project_id and crew_member_id
   - Record check-out timestamp and notes in an audit log table
   - Return updated item
5. Implement CheckIn RPC:
   - Validate item exists and is in 'checked_out' status
   - Update item status to 'available', clear project assignment
   - Allow condition update (e.g., damaged, needs_repair)
   - Record check-in audit entry
6. Implement QueryAvailability RPC that returns items available for a given date range, category, and location.
7. Add migration for inventory_audit_log table (item_id, action, project_id, crew_member_id, timestamp, notes).
8. Register service with gRPC server.

## Validation
CheckOut transitions item from 'available' to 'checked_out' and creates audit record; CheckOut on already-checked-out item returns error; CheckIn transitions item back to 'available'; GetItemByBarcode returns correct item; QueryAvailability excludes checked-out items for requested date range; audit log entries are created for all check-in/check-out operations.