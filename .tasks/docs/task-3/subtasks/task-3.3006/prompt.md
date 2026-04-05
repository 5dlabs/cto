Implement subtask 3006: Implement InventoryService with barcode scanning logic

## Objective
Implement gRPC handlers for InventoryService including CRUD operations, barcode-based lookup, and checkout/return workflows that track inventory assignment to projects.

## Steps
1. Implement InventoryService handlers in /internal/inventory/service.go: CreateItem (generate or accept barcode, validate uniqueness), GetItem, ListItems (filter by status, category, availability). 2. Implement ScanBarcode RPC: look up inventory_items by barcode field, return item details and current assignment status (which project it's checked out to, if any). 3. Implement CheckoutItems RPC: accept project_id and list of item_ids, validate all items have status 'available', begin transaction, insert project_inventory records, update item status to 'rented', commit. 4. Implement ReturnItems RPC: accept project_id and list of item_ids, validate items are assigned to that project, begin transaction, update project_inventory return_date, set item status back to 'available', commit. 5. Use proper gRPC error codes for items not found, already checked out, or not assigned to the specified project.

## Validation
Unit tests verify barcode lookup returns correct item; CheckoutItems fails if item is already rented; ReturnItems fails if item not assigned to specified project; integration test runs full checkout→scan→return cycle against real database; >80% coverage.