Implement subtask 3006: Implement InventoryService gRPC handlers with PostgreSQL integration

## Objective
Implement the InventoryService server with CRUD, reservation, and release operations backed by PostgreSQL.

## Steps
1. Create `internal/service/inventory/service.go` implementing InventoryServiceServer. 2. Implement CreateItem: validate input (SKU uniqueness), insert into `rms.inventory_items`. 3. Implement GetItem: query by ID. 4. Implement ListItems: pagination, filtering by category, location, availability. 5. Implement UpdateItem: partial updates for item details. 6. Implement ReserveItems: transactionally decrement quantity_available and increment quantity_reserved; fail if insufficient stock. 7. Implement ReleaseItems: transactionally increment quantity_available and decrement quantity_reserved. 8. Create `internal/repository/inventory_repo.go`. 9. Use SELECT FOR UPDATE or serializable transactions for reservation/release to prevent race conditions.

## Validation
Unit tests cover all RPCs; integration tests verify reservation atomicity (concurrent reservations don't over-allocate); ReserveItems with insufficient stock returns FAILED_PRECONDITION; ReleaseItems restores quantities correctly.