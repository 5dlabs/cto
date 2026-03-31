Implement subtask 3005: Define protobuf schemas for InventoryService

## Objective
Write .proto file for InventoryService including barcode lookup, availability checking, and inventory CRUD operations with grpc-gateway annotations.

## Steps
1. Create `proto/rms/v1/inventory.proto`:
   - Messages: InventoryItem, CreateInventoryItemRequest/Response, GetInventoryItemRequest/Response, ListInventoryItemsRequest/Response, UpdateInventoryItemRequest/Response, LookupByBarcodeRequest/Response, CheckAvailabilityRequest/Response, BookItemRequest/Response, ReturnItemRequest/Response
   - Enums: InventoryStatus (AVAILABLE, RENTED, MAINTENANCE, RETIRED), ItemCategory
   - RPCs: CreateInventoryItem, GetInventoryItem, ListInventoryItems, UpdateInventoryItem, LookupByBarcode, CheckAvailability, BookItem, ReturnItem
   - HTTP annotations: POST /api/v1/inventory, GET /api/v1/inventory/{id}, GET /api/v1/inventory/barcode/{barcode}, POST /api/v1/inventory/{id}/book, POST /api/v1/inventory/{id}/return
2. Run `make proto-gen` and verify generated Go code compiles.

## Validation
Proto file compiles without errors. Generated Go code compiles. All RPCs have correct HTTP annotations. Barcode lookup and availability check messages include all necessary fields.