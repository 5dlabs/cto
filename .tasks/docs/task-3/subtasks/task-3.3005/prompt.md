Implement subtask 3005: Implement InventoryService with barcode scanning and check-in/check-out logic

## Objective
Build the InventoryService with barcode-based item lookup, atomic check-out/check-in transactions linked to projects and crew members, and inventory status tracking.

## Steps
1. Create `internal/inventory/` package with repository, service, and handler layers.
2. Implement `repository.go`: CreateItem, GetItemByID, GetItemByBarcode, ListItems (with filters for status, category, location), CreateTransaction, ListTransactionsByItem, ListTransactionsByProject.
3. Implement check-out flow: (a) look up item by barcode or ID, (b) verify item status is 'available', (c) within a transaction — create inventory_transaction(type='check_out'), update item status to 'checked_out', set current_project_id. Return error if item already checked out.
4. Implement check-in flow: (a) verify item status is 'checked_out', (b) within a transaction — create inventory_transaction(type='check_in'), update item status to 'available', clear current_project_id, optionally record condition notes.
5. Implement ScanBarcode RPC: look up item by barcode string, return item details and current status.
6. Add audit trail: every transaction records who, when, which project.
7. Wire up gRPC handlers.

## Validation
Unit tests for status validation (cannot check out already checked-out item, cannot check in available item). Integration tests: create item, scan barcode to verify lookup, check out item to a project, verify status changes, check in item, verify full transaction history. Verify concurrent check-out attempts for the same item result in one success and one failure.