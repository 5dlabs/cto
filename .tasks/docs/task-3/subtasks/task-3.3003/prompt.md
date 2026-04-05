Implement subtask 3003: Define InventoryService, CrewService, and DeliveryService protobuf schemas with grpc-gateway annotations

## Objective
Author `inventory.proto`, `crew.proto`, and `delivery.proto` in `proto/sigma1/rms/v1/` with all RPCs, messages, and REST annotations.

## Steps
1. Create `inventory.proto`:
   - Messages: StockLevel (inventory_item_id, available, reserved, total), GetStockLevelRequest, GetStockLevelResponse, RecordTransactionRequest (inventory_item_id, type enum CHECKOUT/CHECKIN/TRANSFER, project_id optional, from_store_id, to_store_id, user_id), RecordTransactionResponse, ScanBarcodeRequest (barcode string), ScanBarcodeResponse (resolved inventory_item_id, item details)
   - RPCs: GetStockLevel (GET /api/v1/inventory/{inventory_item_id}/stock), RecordTransaction (POST /api/v1/inventory/transactions), ScanBarcode (POST /api/v1/inventory/scan)
2. Create `crew.proto`:
   - Messages: CrewMember (all DB fields), CrewAssignment (all DB fields), ListCrewRequest (pagination), ListCrewResponse, AssignCrewRequest (project_id, crew_member_id, role, start_time, end_time), AssignCrewResponse, ScheduleCrewRequest (project_id, list of assignments), ScheduleCrewResponse
   - RPCs: ListCrew (GET /api/v1/crew), AssignCrew (POST /api/v1/crew/assign), ScheduleCrew (POST /api/v1/crew/schedule)
3. Create `delivery.proto`:
   - Enums: DeliveryStatus (SCHEDULED, IN_TRANSIT, DELIVERED, CANCELLED)
   - Messages: Delivery (all DB fields), ScheduleDeliveryRequest, ScheduleDeliveryResponse, UpdateDeliveryStatusRequest, UpdateDeliveryStatusResponse, OptimizeRouteRequest (list of delivery_ids), OptimizeRouteResponse (v1: returns deliveries in original order with a note that optimization is not yet implemented)
   - RPCs: ScheduleDelivery (POST /api/v1/deliveries), UpdateDeliveryStatus (PATCH /api/v1/deliveries/{id}/status), OptimizeRoute (POST /api/v1/deliveries/optimize)
4. Run `buf lint` and `buf generate`. Verify all Go code compiles.

## Validation
Run `buf lint` with zero errors across all three proto files. Run `buf generate` and verify all generated Go files compile. Confirm each proto has correct google.api.http annotations by inspecting generated gateway files.