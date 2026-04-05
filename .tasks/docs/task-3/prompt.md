Implement task 3: Develop RMS Service (Grizz - Go/gRPC)

## Goal
Build the Rental Management System (RMS) with gRPC and grpc-gateway REST endpoints for opportunities, projects, inventory, crew, and delivery management.

## Task Context
- Agent owner: Grizz
- Stack: Go 1.22+/gRPC
- Priority: high
- Dependencies: 1

## Implementation Plan
{"steps": ["Initialize Go project with gRPC and grpc-gateway, connect to PostgreSQL and Redis using ConfigMap.", "Define protobufs for OpportunityService, ProjectService, InventoryService, CrewService, DeliveryService as per PRD.", "Generate Go code from protobufs.", "Implement business logic for quote-to-project workflow, barcode scanning, crew scheduling, and delivery tracking.", "Integrate Google Calendar API for project events.", "Expose REST endpoints via grpc-gateway.", "Add Prometheus metrics and health checks.", "Write database migrations for RMS schema."]}

## Acceptance Criteria
gRPC and REST endpoints are accessible and return correct data; quote-to-project flow works end-to-end; barcode scan and crew assignment tested; Google Calendar integration creates events; >80% code coverage on service logic.

## Subtasks
- Initialize Go project with gRPC, grpc-gateway, and database connectivity: Set up the Go module, import gRPC and grpc-gateway dependencies, configure PostgreSQL and Redis connections using environment variables from the infra ConfigMap, and establish the project directory structure for all five services.
- Write database migrations for RMS schema: Create SQL migration files for all RMS domain tables: opportunities, projects, inventory items, crew members, crew assignments, deliveries, and supporting lookup/junction tables.
- Define protobuf schemas for all five RMS services: Write .proto files for OpportunityService, ProjectService, InventoryService, CrewService, and DeliveryService, including all request/response messages, enums, and grpc-gateway HTTP annotations.
- Generate Go code from protobuf definitions: Configure protoc/buf for Go code generation including gRPC stubs, grpc-gateway reverse proxies, and OpenAPI specs from the .proto files.
- Implement OpportunityService and ProjectService with quote-to-project workflow: Implement the gRPC handlers for OpportunityService and ProjectService, including the critical ConvertToProject business logic that transitions an accepted quote into an active project.
- Implement InventoryService with barcode scanning logic: Implement gRPC handlers for InventoryService including CRUD operations, barcode-based lookup, and checkout/return workflows that track inventory assignment to projects.
- Implement CrewService with scheduling logic: Implement gRPC handlers for CrewService including crew member CRUD, availability checking, and crew-to-project scheduling with conflict detection.
- Integrate Google Calendar API for crew scheduling events: Add Google Calendar integration to CrewService so that when crew members are scheduled to projects, calendar events are created/updated/deleted on their linked Google Calendar.
- Implement DeliveryService with tracking: Implement gRPC handlers for DeliveryService including delivery CRUD, status tracking, and delivery state machine transitions.
- Wire grpc-gateway REST endpoints, Prometheus metrics, and health checks: Complete the REST gateway configuration, add Prometheus instrumentation to all gRPC handlers, and implement readiness/liveness health check endpoints.
- Write integration tests for end-to-end RMS workflows: Create integration tests that exercise the full quote-to-project workflow, barcode scanning, crew scheduling, and delivery tracking across all five services using a real database.

## Deliverables
- Update the relevant code, configuration, and tests.
- Keep artifacts aligned with the acceptance criteria.
- Document blockers or assumptions in your final summary.