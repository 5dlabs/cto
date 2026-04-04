## Develop Rental Management System (Grizz - Go/gRPC)

### Objective
Build the full RMS service replacing Current RMS — opportunities (quotes), projects, inventory transactions, crew scheduling, and delivery management. Implements gRPC services with grpc-gateway for REST, Google Calendar integration, and conflict detection.

### Ownership
- Agent: grizz
- Stack: Go 1.22+/gRPC
- Priority: high
- Status: pending
- Dependencies: 1

### Implementation Details
1. Initialize Go module `github.com/sigma1/rms` with Go 1.22+.
2. Define protobuf files in `proto/` directory for all 5 gRPC services per PRD:
   - `opportunity.proto` — CreateOpportunity, GetOpportunity, UpdateOpportunity, ListOpportunities, ScoreLead
   - `project.proto` — CreateProject, GetProject, UpdateProject, CheckOut, CheckIn
   - `inventory.proto` — GetStockLevel, RecordTransaction, ScanBarcode
   - `crew.proto` — ListCrew, AssignCrew, ScheduleCrew
   - `delivery.proto` — ScheduleDelivery, UpdateDeliveryStatus, OptimizeRoute
   Use `buf` for protobuf management and code generation.
3. Configure grpc-gateway annotations in proto files for all REST endpoints per PRD:
   - `/api/v1/opportunities`, `/api/v1/projects`, `/api/v1/inventory/transactions`, `/api/v1/crew`, `/api/v1/deliveries/*`
   - Include `POST /api/v1/opportunities/:id/approve` and `POST /api/v1/opportunities/:id/convert`
4. Database layer:
   - Use `pgx/v5` for PostgreSQL, connecting via PgBouncer URL from `sigma1-infra-endpoints`.
   - Migrations using `golang-migrate`: create tables in `rms` schema — opportunities, projects, project_line_items, inventory_items, inventory_transactions, crew_members, crew_assignments, deliveries, delivery_routes.
   - All tables include `org_id UUID NOT NULL` column for row-level filtering (per D6).
5. Business logic:
   - **Quote-to-Project workflow**: Opportunity status machine (pending → qualified → approved → converted). Converting creates a Project linked by opportunity_id.
   - **Lead scoring**: ScoreLead RPC computes GREEN/YELLOW/RED based on customer vetting data, event size, and payment history. Returns `LeadScore` with breakdown.
   - **Conflict detection**: Before confirming checkout, check equipment availability across overlapping date ranges. Return conflicts with affected project IDs.
   - **Barcode scanning**: ScanBarcode accepts barcode string, returns InventoryItem with current location and status.
   - **Crew scheduling**: Calendar-based assignment with conflict detection against existing assignments.
   - **Google Calendar integration**: Use Google Calendar API to sync project events. OAuth2 service account credentials from K8s Secret.
6. Inter-service auth: Validate API key from `Authorization: Bearer <key>` header against `sigma1-service-api-keys` secret.
7. Health and observability:
   - gRPC health checking protocol (`grpc.health.v1.Health`)
   - REST `/health/live` and `/health/ready` via grpc-gateway
   - Prometheus metrics via `grpc-prometheus` interceptor + `/metrics` endpoint
   - Structured logging with `slog`
8. GDPR endpoint: `DELETE /api/v1/gdpr/customer/:id` — delete opportunities, projects, crew assignments for customer, return structured confirmation.
9. Dockerfile: multi-stage build (golang:1.22 builder → gcr.io/distroless/static-debian12 runtime).
10. Kubernetes Deployment:
    - Namespace: `sigma1`, replicas: 2
    - Ports: 50051 (gRPC), 8081 (REST gateway)
    - `envFrom` sigma1-infra-endpoints ConfigMap
    - Secret refs for DB credentials, Google Calendar API, service API keys
    - Liveness/readiness probes
11. Generate OpenAPI spec from grpc-gateway annotations using `protoc-gen-openapiv2`.

### Subtasks
- [ ] Initialize Go module and buf protobuf toolchain: Set up the Go module, directory structure, and buf configuration for protobuf management and code generation across all 5 services.
- [ ] Define opportunity.proto and project.proto with grpc-gateway annotations: Create protobuf definitions for the Opportunity and Project gRPC services with full REST gateway annotations for all endpoints including approve and convert actions.
- [ ] Define inventory.proto, crew.proto, and delivery.proto with grpc-gateway annotations: Create protobuf definitions for the Inventory, Crew, and Delivery gRPC services with full REST gateway annotations.
- [ ] Database migrations for all RMS schema tables: Create golang-migrate migration files for all 9 tables in the rms schema with org_id column, indexes, and foreign key constraints.
- [ ] Implement database repository layer with pgx: Build the Go repository layer using pgx/v5 for all RMS entities with org_id-scoped queries, providing CRUD operations consumed by gRPC service implementations.
- [ ] Implement Opportunity service with state machine, lead scoring, and convert-to-project: Build the OpportunityService gRPC implementation with the full quote-to-project workflow including status state machine, lead scoring algorithm, and opportunity-to-project conversion.
- [ ] Implement Inventory service with conflict detection and barcode scanning: Build the InventoryService gRPC implementation with stock level tracking, transaction recording, equipment availability conflict detection, and barcode scanning lookup.
- [ ] Implement Crew scheduling service with conflict detection: Build the CrewService gRPC implementation with crew listing, assignment, scheduling with overlap conflict detection, and availability queries.
- [ ] Implement Google Calendar integration for crew/project sync: Build Google Calendar API integration to sync project events and crew assignments to Google Calendar using OAuth2 service account credentials.
- [ ] Implement Delivery management service: Build the DeliveryService gRPC implementation with delivery scheduling, status updates, route optimization, and listing.
- [ ] Implement inter-service auth, GDPR endpoint, and gRPC server bootstrap: Build the API key authentication interceptor, GDPR customer deletion endpoint, and the main gRPC + grpc-gateway server wiring.
- [ ] Implement health checks, Prometheus metrics, and structured logging: Add gRPC health checking protocol, REST health endpoints, Prometheus metrics via grpc-prometheus, and structured logging with slog throughout the service.
- [ ] Create Dockerfile and Kubernetes deployment manifests: Build the multi-stage Dockerfile and Kubernetes manifests for deploying the RMS service in the sigma1 namespace with proper configuration, secrets, and probes.
- [ ] Generate and validate OpenAPI spec from grpc-gateway annotations: Generate the OpenAPI v2 specification from protobuf grpc-gateway annotations and validate it for correctness and completeness.