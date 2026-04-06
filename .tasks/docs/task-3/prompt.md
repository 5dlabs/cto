Implement task 3: Develop RMS Service (Grizz - Go/gRPC)

## Goal
Build the Rental Management System backend with gRPC and grpc-gateway REST endpoints for opportunities, projects, inventory, crew, and deliveries.

## Task Context
- Agent owner: grizz
- Stack: Go 1.22+/gRPC
- Priority: high
- Dependencies: 1

## Implementation Plan
{"steps": ["Initialize Go project with gRPC and grpc-gateway setup, using PostgreSQL and Redis from 'sigma1-infra-endpoints'", "Define protobufs for OpportunityService, ProjectService, InventoryService, CrewService, DeliveryService as per PRD", "Implement REST endpoints via grpc-gateway for all listed routes", "Integrate Google Calendar API for project scheduling", "Implement barcode scanning logic for check-in/check-out", "Add conflict detection for bookings and inventory", "Write unit and integration tests for all gRPC and REST endpoints", "Document API usage and authentication requirements"]}

## Acceptance Criteria
gRPC and REST endpoints return correct data and status codes; barcode scanning updates inventory; Google Calendar integration creates events; conflict detection prevents double-booking; all endpoints covered by tests with >80% code coverage.

## Subtasks
- Scaffold Go project with gRPC, grpc-gateway, and infrastructure connectivity: Initialize the Go module with gRPC server, grpc-gateway HTTP proxy, PostgreSQL connection pool, and Redis client, all reading connection strings from the sigma1-infra-endpoints ConfigMap. Include health check and readiness endpoints.
- Define protobuf schemas for OpportunityService and ProjectService: Author .proto files for OpportunityService and ProjectService with all RPC methods, request/response messages, and grpc-gateway HTTP annotations as specified in the PRD.
- Define protobuf schemas for InventoryService: Author .proto files for InventoryService covering equipment items, availability tracking, barcode associations, and check-in/check-out operations.
- Define protobuf schemas for CrewService and DeliveryService: Author .proto files for CrewService (crew member management, assignments, availability) and DeliveryService (delivery scheduling, tracking, status updates).
- Implement database migrations and repository layer for all RMS entities: Create PostgreSQL schema migrations for opportunities, projects, inventory items, crew members, assignments, and deliveries. Implement a repository layer with CRUD operations using pgx.
- Implement OpportunityService and ProjectService gRPC handlers: Implement the gRPC server handlers for OpportunityService and ProjectService, wiring protobuf RPCs to the repository layer with proper validation and error handling.
- Implement InventoryService gRPC handlers with barcode scanning logic: Implement InventoryService gRPC handlers including barcode-based lookup, check-in/check-out workflows that update item status and create audit records.
- Implement CrewService and DeliveryService gRPC handlers with conflict detection: Implement CrewService and DeliveryService handlers including crew assignment conflict detection that prevents double-booking crew members for overlapping date ranges.
- Integrate Google Calendar API for project scheduling: Implement Google Calendar API integration within ProjectService to create, update, and delete calendar events when projects are created or modified.
- Write integration tests for all RMS gRPC and REST endpoints: Create comprehensive integration tests covering all 5 services via both gRPC and REST interfaces, targeting >80% code coverage.

## Deliverables
- Update the relevant code, configuration, and tests.
- Keep artifacts aligned with the acceptance criteria.
- Document blockers or assumptions in your final summary.