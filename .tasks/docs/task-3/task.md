## Develop Rental Management System (Grizz - Go/gRPC)

### Objective
Build the RMS backend with gRPC and grpc-gateway REST endpoints for opportunity, project, inventory, crew, and delivery management, supporting quote-to-project workflow and barcode scanning.

### Ownership
- Agent: Grizz
- Stack: Go/gRPC
- Priority: high
- Status: pending
- Dependencies: 1

### Implementation Details
{"steps": ["Initialize Go 1.22+ project with gRPC and grpc-gateway setup.", "Define protobufs for OpportunityService, ProjectService, InventoryService, CrewService, DeliveryService as per PRD.", "Implement REST endpoints via grpc-gateway.", "Integrate with PostgreSQL for all RMS data (use connection string from ConfigMap).", "Use Redis for session cache.", "Implement barcode scanning logic and inventory transaction recording.", "Integrate with Google Calendar API for project scheduling.", "Implement lead scoring and conflict detection logic.", "Add health and metrics endpoints.", "Document gRPC and REST API usage."]}

### Subtasks
- [ ] Initialize Go project with gRPC and grpc-gateway scaffolding: Set up the Go 1.22+ module with gRPC server, grpc-gateway reverse proxy, project directory structure, build tooling, and base configuration loading from ConfigMap environment variables.
- [ ] Define protobuf schemas for all five RMS services: Author .proto files for OpportunityService, ProjectService, InventoryService, CrewService, and DeliveryService with all message types, RPC methods, and grpc-gateway HTTP annotations.
- [ ] Implement PostgreSQL schema migrations and repository layer: Create SQL migration files for all RMS domain tables and implement a Go repository layer for database access using the PostgreSQL connection from ConfigMap.
- [ ] Implement OpportunityService with lead scoring and conflict detection: Build the OpportunityService gRPC handler with full CRUD, lead scoring algorithm (GREEN/YELLOW/RED), date/equipment conflict detection, and ConvertToProject RPC.
- [ ] Implement ProjectService with quote-to-project workflow: Build the ProjectService gRPC handler supporting project lifecycle, quote generation, quote approval, and status transitions.
- [ ] Implement InventoryService with barcode scanning and transaction recording: Build the InventoryService gRPC handler supporting barcode scanning, check-in/check-out workflows, and inventory transaction history.
- [ ] Implement CrewService and DeliveryService: Build the CrewService and DeliveryService gRPC handlers for crew assignment, availability management, delivery scheduling, and status tracking.
- [ ] Integrate Google Calendar API for project scheduling: Implement Google Calendar integration to sync project schedules, crew assignments, and delivery events to a shared calendar.
- [ ] Integrate Redis session cache: Set up Redis client connection using the ConfigMap URL and implement session caching for the RMS service.
- [ ] Add health, metrics, and API documentation endpoints: Implement Prometheus metrics collection, health/readiness probes, and generate API documentation for the RMS gRPC and REST endpoints.