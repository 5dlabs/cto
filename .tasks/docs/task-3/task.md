## Develop RMS Service (Grizz - Go/gRPC)

### Objective
Build the Rental Management System (RMS) with gRPC and REST (grpc-gateway) for opportunity, project, inventory, crew, and delivery management. Enables quote-to-project workflow and inventory tracking.

### Ownership
- Agent: grizz
- Stack: Go 1.22+, gRPC
- Priority: high
- Status: pending
- Dependencies: 1

### Implementation Details
{"steps": ["Initialize Go project with gRPC and grpc-gateway setup.", "Connect to PostgreSQL and Redis using ConfigMap endpoints.", "Define protobufs for OpportunityService, ProjectService, InventoryService, CrewService, DeliveryService as per PRD.", "Implement REST endpoints via grpc-gateway.", "Integrate barcode scanning logic and Google Calendar API for scheduling.", "Implement conflict detection for bookings.", "Add Prometheus metrics and health checks.", "Seed database with sample opportunities, projects, and inventory."]}

### Subtasks
- [ ] Scaffold Go project with gRPC server and grpc-gateway: Initialize the Go module, set up the gRPC server entrypoint, configure grpc-gateway for REST proxy, and establish the project directory structure following idiomatic Go layout for a multi-service gRPC application.
- [ ] Configure PostgreSQL database connectivity and migration framework: Establish PostgreSQL connection pooling using ConfigMap endpoints, set up a migration framework (golang-migrate or goose), and create the initial RMS schema migrations for all domain tables.
- [ ] Configure Redis connectivity: Establish Redis client connection using ConfigMap endpoints for use in conflict detection caching and session-related lookups.
- [ ] Define protobuf schemas for OpportunityService and ProjectService: Write .proto files for OpportunityService and ProjectService including all message types, enums, RPC methods, and grpc-gateway HTTP annotations for the quote-to-project workflow.
- [ ] Define protobuf schemas for InventoryService: Write .proto file for InventoryService including barcode lookup, availability checking, and inventory CRUD operations with grpc-gateway annotations.
- [ ] Define protobuf schemas for CrewService and DeliveryService: Write .proto files for CrewService (crew management and scheduling) and DeliveryService (delivery/pickup tracking) with grpc-gateway annotations.
- [ ] Implement OpportunityService gRPC handlers with quote-to-project conversion: Implement the OpportunityService gRPC server including CRUD operations and the ConvertOpportunityToProject RPC that transitions an opportunity into a project (the core quote-to-project workflow).
- [ ] Implement ProjectService gRPC handlers: Implement the ProjectService gRPC server with CRUD operations for projects, including status management and querying by opportunity linkage.
- [ ] Implement InventoryService gRPC handlers with barcode scanning and conflict detection: Implement the InventoryService gRPC server including CRUD, barcode lookup, availability checking with conflict detection for overlapping bookings, and book/return operations.
- [ ] Implement CrewService gRPC handlers with Google Calendar API integration: Implement the CrewService gRPC server for crew member management, project assignment, and availability checking integrated with Google Calendar API for scheduling.
- [ ] Implement DeliveryService gRPC handlers: Implement the DeliveryService gRPC server for scheduling and tracking deliveries and pickups associated with projects.
- [ ] Register all services with grpc-gateway and expose REST endpoints: Wire all five gRPC services into the grpc-gateway HTTP mux so that REST endpoints are automatically exposed based on proto annotations.
- [ ] Add Prometheus metrics and health check endpoints: Instrument the RMS service with Prometheus metrics for request counts, latencies, and error rates, and add gRPC health check and HTTP readiness/liveness probes.
- [ ] Seed database with sample RMS data: Create a database seeding script or Go command that populates the RMS database with sample opportunities, projects, inventory items, crew members, bookings, and deliveries for development and testing.