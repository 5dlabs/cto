Implement subtask 3005: Implement Inventory, Crew, and Delivery gRPC service logic

## Objective
Implement the server-side logic for `InventoryService`, `CrewService`, and `DeliveryService`, including inventory transactions, crew scheduling, and delivery tracking.

## Steps
1. Implement gRPC handlers for `RecordInventoryTransaction`, `GetInventoryTransactionsByProject`.2. Implement gRPC handlers for `CrewService` (e.g., `ScheduleCrew`, `GetCrewAvailability`).3. Implement gRPC handlers for `DeliveryService` (e.g., `CreateDelivery`, `TrackDelivery`).

## Validation
1. Use `grpcurl` to test `RecordInventoryTransaction`, `ScheduleCrew`, and `CreateDelivery` endpoints.2. Verify data consistency in PostgreSQL for inventory, crew, and delivery entities.