Implement subtask 3009: Implement InventoryService gRPC handlers with barcode scanning and conflict detection

## Objective
Implement the InventoryService gRPC server including CRUD, barcode lookup, availability checking with conflict detection for overlapping bookings, and book/return operations.

## Steps
1. Create `/internal/service/inventory_service.go` implementing the generated InventoryServiceServer interface.
2. Implement CreateInventoryItem, GetInventoryItem, ListInventoryItems, UpdateInventoryItem as standard CRUD.
3. Implement LookupByBarcode: query `inventory_items` by barcode field, return the item or NOT_FOUND.
4. Implement CheckAvailability:
   - Accept item_id and date range.
   - Query `bookings` table for overlapping date ranges on this item.
   - Return availability status and any conflicting bookings.
   - Optionally cache availability checks in Redis with short TTL for frequently queried items.
5. Implement BookItem:
   - Validate item exists and is AVAILABLE.
   - Run conflict detection: check for overlapping bookings in the requested date range.
   - If conflict found, return ALREADY_EXISTS error with conflict details.
   - Use SELECT FOR UPDATE or advisory lock to prevent race conditions.
   - Insert booking record, update item status if immediately rented.
6. Implement ReturnItem:
   - Update booking status to completed.
   - Update item status back to AVAILABLE.
7. Create `/internal/repository/inventory_repo.go` and `/internal/repository/booking_repo.go`.
8. Register the service with the gRPC server.

## Validation
Unit tests with mock repos for all RPCs. Integration tests: create item, book it for a date range, attempt overlapping booking (expect conflict error), return item and verify re-availability. Barcode lookup returns correct item. Concurrent booking attempts on the same item/dates — only one succeeds.