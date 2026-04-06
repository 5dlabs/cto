Implement task 3: Develop Rental Management System (Grizz - Go/gRPC)

## Goal
Build the RMS service with gRPC and grpc-gateway REST endpoints for opportunity, project, inventory, crew, and delivery management. Integrate with PostgreSQL, Redis, and Google Calendar.

## Task Context
- Agent owner: Grizz
- Stack: Go 1.22+/gRPC
- Priority: high
- Dependencies: 1

## Implementation Plan
{"steps": ["Initialize Go project with gRPC and grpc-gateway.", "Define protobufs for OpportunityService, ProjectService, InventoryService, CrewService, DeliveryService as per PRD.", "Generate Go code and implement service logic.", "Integrate with PostgreSQL for all RMS data (schema: rms).", "Use Redis for session cache.", "Implement Google Calendar API integration for project/crew scheduling.", "Expose REST endpoints via grpc-gateway as specified.", "Connect to infra via envFrom: sigma1-infra-endpoints ConfigMap.", "Implement health and metrics endpoints."]}

## Acceptance Criteria
All gRPC and REST endpoints are reachable and return correct data; Google Calendar integration works for scheduling; health and metrics endpoints are available; quote-to-project workflow completes in <2 minutes.

## Subtasks
- Initialize Go project with gRPC and grpc-gateway scaffolding: Set up the Go module, directory structure, gRPC server bootstrap, grpc-gateway reverse proxy, and ConfigMap-based configuration loading for the RMS service.
- Define protobuf schemas for all five RMS services: Author .proto files for OpportunityService, ProjectService, InventoryService, CrewService, and DeliveryService including all message types, enums, and gRPC method definitions with grpc-gateway HTTP annotations.
- Create PostgreSQL schema migrations for the RMS schema: Write SQL migration files for all RMS tables in the `rms` PostgreSQL schema, including opportunities, projects, inventory_items, crew_members, deliveries, and join tables.
- Implement OpportunityService gRPC handlers with PostgreSQL integration: Implement the OpportunityService server including all CRUD operations and the ConvertToProject workflow, backed by PostgreSQL queries.
- Implement ProjectService gRPC handlers with PostgreSQL integration: Implement the ProjectService server with CRUD operations and crew assignment, backed by PostgreSQL.
- Implement InventoryService gRPC handlers with PostgreSQL integration: Implement the InventoryService server with CRUD, reservation, and release operations backed by PostgreSQL.
- Implement DeliveryService gRPC handlers with PostgreSQL integration: Implement the DeliveryService server with CRUD and status tracking, backed by PostgreSQL.
- Implement CrewService gRPC handlers with PostgreSQL integration: Implement the CrewService server with CRUD and availability checking, backed by PostgreSQL. This subtask covers only the database-backed operations, not the Google Calendar integration.
- Integrate Google Calendar API for crew and project scheduling: Implement the Google Calendar API client and wire it into CrewService.ScheduleCrew and ProjectService for calendar event creation and availability checking.
- Implement Redis session cache integration: Add Redis client initialization and session caching logic to the RMS service for caching frequently accessed data and session state.
- Expose REST endpoints via grpc-gateway and implement health/metrics: Configure grpc-gateway to serve all five RMS services as REST endpoints, add Prometheus metrics instrumentation, and implement health/readiness endpoints.
- Write end-to-end integration tests for quote-to-project workflow: Create integration tests that exercise the full quote-to-project lifecycle spanning OpportunityService, ProjectService, CrewService, InventoryService, and DeliveryService.

## Deliverables
- Update the relevant code, configuration, and tests.
- Keep artifacts aligned with the acceptance criteria.
- Document blockers or assumptions in your final summary.