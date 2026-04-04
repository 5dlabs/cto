Implement subtask 3007: Implement Inventory service with conflict detection and barcode scanning

## Objective
Build the InventoryService gRPC implementation with stock level tracking, transaction recording, equipment availability conflict detection, and barcode scanning lookup.

## Steps
1. Create `internal/service/inventory_svc.go` implementing `InventoryServiceServer`.
2. Implement `GetStockLevel` RPC: delegate to `inventoryRepo.GetStockLevel()`, return item with quantity_total and quantity_available.
3. Implement `RecordTransaction` RPC:
   - Validate transaction type (CHECK_OUT, CHECK_IN, TRANSFER, ADJUSTMENT).
   - For CHECK_OUT: verify quantity_available >= requested quantity, decrement atomically.
   - For CHECK_IN: increment quantity_available, update item status.
   - Delegate to `inventoryRepo.RecordTransaction()` which uses pgx transaction.
4. Implement conflict detection in `internal/domain/conflict_detector.go`:
   - `DetectConflicts(ctx, orgID, itemIDs []UUID, dateStart, dateEnd time.Time) ([]Conflict, error)`
   - Query inventory_transactions joined with projects to find overlapping CHECK_OUT periods for the same items.
   - Return list of Conflict structs with item_id, conflicting_project_id, conflicting_date_range.
5. Integrate conflict detection into Project service's `CheckOut` RPC: before recording transactions, call DetectConflicts. If conflicts found, return them in CheckOutResponse with success=false.
6. Implement `ScanBarcode` RPC: call `inventoryRepo.GetByBarcode(ctx, orgID, barcode)`, return InventoryItem with current_location and status. Return NOT_FOUND if barcode doesn't exist.
7. Implement `ListInventoryItems` with org_id filtering and pagination.
8. Register service in gRPC server.

## Validation
Integration tests with testcontainers-go: 1) Create inventory item, RecordTransaction CHECK_OUT, verify quantity_available decremented. RecordTransaction CHECK_IN, verify quantity_available restored. 2) Conflict detection: create item, check out to project A for dates Jan 1-5, attempt checkout to project B for Jan 3-7, verify conflict returned with project A's ID and overlapping range. 3) ScanBarcode: create item with barcode 'ABC123', scan returns correct item; scan non-existent barcode returns NOT_FOUND. 4) Verify CHECK_OUT fails when quantity_available < requested.