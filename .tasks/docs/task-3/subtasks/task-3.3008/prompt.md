Implement subtask 3008: Implement InventoryService gRPC handlers with stock aggregation and barcode scan

## Objective
Implement the InventoryService gRPC server including stock level computation from transaction history and barcode-based item lookup.

## Steps
1. Create `internal/service/inventory.go` implementing InventoryServiceServer.
2. Implement GetStockLevel:
   - Query inventory_transactions for given inventory_item_id
   - Aggregate: available = SUM(checkin quantities) - SUM(checkout quantities) + initial stock
   - Return StockLevel with available, reserved (checked out to active projects), total
3. Implement RecordTransaction:
   - Validate transaction type (checkout, checkin, transfer)
   - For checkout: verify sufficient stock, record transaction
   - For checkin: record transaction
   - For transfer: record with from_store_id and to_store_id, ensure both are set
   - Return transaction ID and updated stock level
4. Implement ScanBarcode:
   - Accept barcode string
   - Lookup in inventory system (v1: simple table lookup, barcode column on inventory_items or a barcode_mappings table)
   - Return resolved inventory_item_id with item details (name, category, current stock level)
   - Return NOT_FOUND if barcode unrecognized
5. All operations use proper gRPC error codes.
6. Register service in gRPC server.

## Validation
Unit test stock aggregation logic with various transaction combinations. Integration test: record checkout, verify stock decreases; record checkin, verify stock increases. Barcode scan test: insert a known barcode mapping, scan it, verify correct inventory_item_id is returned. Test ScanBarcode with unknown barcode returns NOT_FOUND.