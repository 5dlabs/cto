Implement subtask 3006: Implement InventoryService with barcode scanning and transaction recording

## Objective
Build the InventoryService gRPC handler supporting barcode scanning, check-in/check-out workflows, and inventory transaction history.

## Steps
1. Implement InventoryService gRPC server in /internal/inventory/. 2. Wire up ListInventory, GetItem RPCs for inventory browsing. 3. Implement ScanBarcode RPC: accept a barcode string, look up the inventory item, return current status and details. 4. Implement CheckOut RPC: validate item is available, associate with a project, record an inventory_transaction (type: checkout, timestamp, project_id, user_id), update item status to 'checked_out'. 5. Implement CheckIn RPC: validate item is checked out, update status to 'available', record inventory_transaction (type: checkin), optionally record condition notes. 6. Implement RecordTransaction RPC for manual transaction entries (damage, maintenance, etc.). 7. Register service and verify REST routes.

## Validation
Barcode scan returns correct item details; check-out changes item status and creates transaction record; check-in restores availability; attempting to check out an already-checked-out item returns an error; transaction history is accurate and ordered.