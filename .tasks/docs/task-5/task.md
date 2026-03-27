## RMS Service - Inventory, Crew, Delivery gRPC & REST (Grizz - Go/gRPC)

### Objective
Expand the RMS service to include Inventory, Crew, and Delivery management. This involves defining new protobuf services, implementing their logic, and extending the REST gateway.

### Ownership
- Agent: Grizz
- Stack: Go/gRPC
- Priority: medium
- Status: pending
- Dependencies: 1, 4

### Implementation Details
1. Define protobuf schemas for `InventoryService`, `CrewService`, and `DeliveryService`, including RPCs like `GetStockLevel`, `RecordTransaction`, `ScanBarcode`, `ListCrew`, `AssignCrew`, `ScheduleCrew`, `ScheduleDelivery`, `UpdateDeliveryStatus`, `OptimizeRoute`. 2. Generate Go code from new protobuf definitions. 3. Implement the gRPC server for these new services. 4. Extend the PostgreSQL schema to support `InventoryTransaction`, `CrewMember`, `Delivery` models. 5. Implement the logic for each RPC, including database interactions for inventory transactions, crew assignments, and delivery scheduling. 6. Extend `grpc-gateway` to expose REST endpoints for these services (e.g., `GET /api/v1/inventory/transactions`, `POST /api/v1/inventory/transactions`, `GET /api/v1/crew`, `POST /api/v1/crew/assign`, `POST /api/v1/deliveries/schedule`). 7. Implement a stub for Google Calendar API integration for crew scheduling, logging calls without actual external interaction for now. Use Go 1.22+.

### Subtasks
- [ ] Implement RMS Service - Inventory, Crew, Delivery gRPC & REST (Grizz - Go/gRPC): Expand the RMS service to include Inventory, Crew, and Delivery management. This involves defining new protobuf services, implementing their logic, and extending the REST gateway.