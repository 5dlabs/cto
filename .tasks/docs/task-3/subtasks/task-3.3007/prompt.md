Implement subtask 3007: Implement DeliveryService gRPC handlers with PostgreSQL integration

## Objective
Implement the DeliveryService server with CRUD and status tracking, backed by PostgreSQL.

## Steps
1. Create `internal/service/delivery/service.go` implementing DeliveryServiceServer. 2. Implement CreateDelivery: validate project exists, insert into `rms.deliveries`, insert associated items into `rms.delivery_items`. 3. Implement GetDelivery: query by ID with joined delivery items. 4. Implement ListDeliveries: pagination, filtering by project_id, status, date range. 5. Implement UpdateDeliveryStatus: validate status transitions (PENDING→IN_TRANSIT→DELIVERED or RETURNED), update actual_date on delivery/return. 6. Create `internal/repository/delivery_repo.go`. 7. Register service.

## Validation
Unit tests cover all RPCs and status transition validation; integration tests: create delivery with items, update status through lifecycle, verify delivery_items join table; invalid transitions return proper error codes.