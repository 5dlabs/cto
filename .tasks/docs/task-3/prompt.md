Implement task 3: Develop Rental Management System (Grizz - Go/gRPC)

## Goal
Build the RMS backend for opportunity, project, inventory, crew, and delivery management, exposing both gRPC and REST (grpc-gateway) APIs.

## Task Context
- Agent owner: grizz
- Stack: Go/gRPC
- Priority: high
- Dependencies: 1

## Implementation Plan
{"steps":["Initialize Go 1.22+ project with gRPC and grpc-gateway, using POSTGRES_URL and REDIS_URL from ConfigMap.","Define protobuf services: OpportunityService, ProjectService, InventoryService, CrewService, DeliveryService as per PRD.","Implement REST endpoints via grpc-gateway for all core workflows (quote, project, inventory, crew, delivery).","Integrate Google Calendar API for project/crew scheduling.","Implement barcode scanning and inventory transaction logic.","Add conflict detection for bookings.","Expose Prometheus metrics and health endpoints.","Ensure quote-to-project and check-in/check-out flows are atomic and auditable."]}

## Acceptance Criteria
gRPC and REST endpoints are accessible and return correct data. End-to-end quote-to-project and inventory workflows function as described. Google Calendar integration works. Health and metrics endpoints are available.

## Subtasks
- Initialize Go project with gRPC, grpc-gateway, and database migrations: Set up the Go 1.22+ module structure with gRPC server, grpc-gateway reverse proxy, database connection pooling via POSTGRES_URL and REDIS_URL from ConfigMap, and initial database schema migrations for all RMS domain tables (opportunities, projects, inventory_items, inventory_transactions, crew_members, crew_assignments, deliveries).
- Define protobuf service definitions for all five RMS domains: Create .proto files for OpportunityService, ProjectService, InventoryService, CrewService, and DeliveryService with all message types, RPC methods, and grpc-gateway HTTP annotations.
- Implement OpportunityService with quote workflow: Build the OpportunityService gRPC server implementation including CRUD operations for opportunities/quotes, status transitions (draft → sent → accepted → converted), and line item management.
- Implement ProjectService with quote-to-project conversion and Google Calendar integration: Build the ProjectService including atomic quote-to-project conversion (marking opportunity as converted and creating project in a single transaction) and Google Calendar API integration for scheduling.
- Implement InventoryService with barcode scanning and check-in/check-out logic: Build the InventoryService with barcode-based item lookup, atomic check-out/check-in transactions linked to projects and crew members, and inventory status tracking.
- Implement CrewService with scheduling and calendar sync: Build the CrewService for managing crew members, assigning them to projects with date ranges, detecting scheduling conflicts, and syncing assignments to Google Calendar.
- Implement DeliveryService with logistics tracking: Build the DeliveryService for scheduling equipment deliveries and pickups linked to projects, with status tracking through the delivery lifecycle.
- Implement booking conflict detection across all services: Build a cross-cutting conflict detection system that checks for scheduling conflicts across projects, crew assignments, inventory availability, and delivery windows before confirming bookings.
- Add Prometheus metrics and health endpoints: Instrument all RMS services with Prometheus metrics (request counts, latencies, error rates) and expose health/readiness probe endpoints.
- End-to-end workflow validation tests: Write comprehensive end-to-end tests covering the full quote-to-project lifecycle, inventory check-in/check-out workflows, and cross-service interactions.

## Deliverables
- Update the relevant code, configuration, and tests.
- Keep artifacts aligned with the acceptance criteria.
- Document blockers or assumptions in your final summary.