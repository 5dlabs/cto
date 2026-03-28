## Develop Rental Management System (RMS) Service (Grizz - Go/gRPC)

### Objective
Build the core Rental Management System (RMS) service, handling opportunities, projects, inventory, crew, and deliveries. This service is central to the quote-to-project workflow and operational logistics.

### Ownership
- Agent: grizz
- Stack: Go/gRPC
- Priority: high
- Status: pending
- Dependencies: 1

### Implementation Details
1. Initialize a new Go project targeting Go 1.22.2.2. Define gRPC services and protobufs for `OpportunityService`, `ProjectService`, `InventoryService`, `CrewService`, and `DeliveryService`.3. Generate Go code from protobuf definitions.4. Implement gRPC server logic for all services.5. Integrate `grpc-gateway` to expose REST endpoints as defined in the PRD.6. Define `Opportunity`, `Project`, and `InventoryTransaction` data models and implement `sqlx` (or similar ORM) for PostgreSQL interaction, referencing the `sigma1-infra-endpoints` ConfigMap.7. Implement core features: quote-to-project conversion, barcode scanning (mocked initially), crew scheduling, and delivery tracking.8. Integrate with Redis for session caching, referencing the `sigma1-infra-endpoints` ConfigMap.9. Add basic health checks for gRPC and REST endpoints.

### Subtasks
