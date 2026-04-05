Implement task 3: Develop RMS Service (Grizz - Go/gRPC)

## Goal
Build the Rental Management System (RMS) with gRPC and grpc-gateway REST endpoints for opportunity, project, inventory, crew, and delivery management. This is the backbone for quote-to-project workflows.

## Task Context
- Agent owner: grizz
- Stack: Go/gRPC
- Priority: high
- Dependencies: 1

## Implementation Plan
{"steps": ["Initialize Go 1.22+ project with gRPC and grpc-gateway.", "Define protobufs for OpportunityService, ProjectService, InventoryService, CrewService, DeliveryService as per PRD.", "Implement REST endpoints via grpc-gateway for all specified routes.", "Integrate with PostgreSQL for all RMS data and Redis for session cache.", "Implement barcode scanning, crew scheduling, and delivery tracking logic.", "Add Google Calendar API integration for project/crew scheduling.", "Reference connection strings from 'sigma1-infra-endpoints' ConfigMap via envFrom.", "Write unit and integration tests for all gRPC and REST endpoints."]}

## Acceptance Criteria
All gRPC and REST endpoints are reachable and return correct data; barcode scanning and crew scheduling work as specified; Google Calendar integration is functional; tests cover at least 80% of code paths.

## Subtasks
- Scaffold Go project with gRPC, grpc-gateway, and infrastructure config: Initialize a Go 1.22+ module with gRPC server, grpc-gateway HTTP proxy, project directory structure, and infrastructure configuration referencing the sigma1-infra-endpoints ConfigMap.
- Define protobuf schemas for all five RMS services and generate Go code: Author .proto files for OpportunityService, ProjectService, InventoryService, CrewService, and DeliveryService with all message types, enums, and grpc-gateway HTTP annotations as per the PRD.
- Implement PostgreSQL database layer and migrations for RMS schema: Create the database migration files and a shared repository/data-access layer for all five RMS domain tables in the rms PostgreSQL schema.
- Implement OpportunityService and ProjectService gRPC handlers: Build the gRPC service implementations for OpportunityService and ProjectService with full business logic, including the opportunity-to-project conversion workflow.
- Implement InventoryService with barcode scanning logic: Build the InventoryService gRPC handler with barcode scanning, check-out/check-in workflows, and inventory status management.
- Implement CrewService with scheduling logic: Build the CrewService gRPC handler for crew member management and project scheduling, including availability tracking.
- Implement DeliveryService with delivery tracking logic: Build the DeliveryService gRPC handler for scheduling, tracking, and managing equipment deliveries tied to projects.
- Integrate Redis for session cache and ephemeral state: Add Redis (Valkey) integration for session caching, barcode scan deduplication, and ephemeral operational state in the RMS service.
- Integrate Google Calendar API for crew and project scheduling: Add Google Calendar API integration so that crew assignments and project schedules are synced bidirectionally with Google Calendar.
- Write integration tests for all RMS gRPC and REST endpoints: Create comprehensive integration tests covering all five services end-to-end, including the quote-to-project workflow, barcode scanning, and cross-service interactions.

## Deliverables
- Update the relevant code, configuration, and tests.
- Keep artifacts aligned with the acceptance criteria.
- Document blockers or assumptions in your final summary.