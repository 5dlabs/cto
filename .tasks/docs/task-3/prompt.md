Implement task 3: Develop Rental Management System (RMS) Service (Grizz - Go/gRPC)

## Goal
Build the core Rental Management System (RMS) service, handling opportunities, projects, inventory, crew, and deliveries. This service is central to the quote-to-project workflow and operational logistics.

## Task Context
- Agent owner: grizz
- Stack: Go/gRPC
- Priority: high
- Dependencies: 1

## Implementation Plan
1. Initialize a new Go project targeting Go 1.22.2.2. Define gRPC services and protobufs for `OpportunityService`, `ProjectService`, `InventoryService`, `CrewService`, and `DeliveryService`.3. Generate Go code from protobuf definitions.4. Implement gRPC server logic for all services.5. Integrate `grpc-gateway` to expose REST endpoints as defined in the PRD.6. Define `Opportunity`, `Project`, and `InventoryTransaction` data models and implement `sqlx` (or similar ORM) for PostgreSQL interaction, referencing the `sigma1-infra-endpoints` ConfigMap.7. Implement core features: quote-to-project conversion, barcode scanning (mocked initially), crew scheduling, and delivery tracking.8. Integrate with Redis for session caching, referencing the `sigma1-infra-endpoints` ConfigMap.9. Add basic health checks for gRPC and REST endpoints.

## Acceptance Criteria
1. Deploy the service to Kubernetes and verify it starts successfully.2. Use `grpcurl` to test gRPC endpoints for `CreateOpportunity`, `GetProject`, `RecordTransaction`, etc.3. Use `curl` or Postman to verify REST endpoints exposed via `grpc-gateway` are functional.4. Test the full quote-to-project workflow: create opportunity, approve, convert to project.5. Verify inventory transactions can be recorded and retrieved.6. Confirm data persistence in PostgreSQL for all RMS entities.

## Subtasks
- Initialize Go project and define gRPC protobuf schemas: Initialize a new Go project and define all necessary gRPC service and message protobufs for Opportunity, Project, Inventory, Crew, and Delivery services.
- Generate Go code and set up base gRPC server: Generate Go code from the defined protobufs and establish a basic gRPC server structure capable of serving requests.
- Define data models and integrate with PostgreSQL: Define Go structs for `Opportunity`, `Project`, and `InventoryTransaction` and integrate `sqlx` for PostgreSQL interaction, using the `sigma1-infra-endpoints` ConfigMap.
- Implement Opportunity and Project gRPC service logic: Implement the server-side logic for `OpportunityService` and `ProjectService`, including quote-to-project conversion.
- Implement Inventory, Crew, and Delivery gRPC service logic: Implement the server-side logic for `InventoryService`, `CrewService`, and `DeliveryService`, including inventory transactions, crew scheduling, and delivery tracking.
- Integrate grpc-gateway, Redis caching, and health checks: Integrate `grpc-gateway` to expose REST endpoints, add Redis for session caching, and implement basic health checks for gRPC and REST endpoints.

## Deliverables
- Update the relevant code, configuration, and tests.
- Keep artifacts aligned with the acceptance criteria.
- Document blockers or assumptions in your final summary.