Implement task 3: Build Rental Management System Service (Grizz - Go/gRPC)

## Goal
Implement the full Rental Management System replacing Current RMS — opportunities (quotes), projects, inventory transactions, crew scheduling, and delivery management. Exposes gRPC services natively with grpc-gateway REST for external consumers including Morgan's MCP tools.

## Task Context
- Agent owner: grizz
- Stack: Go 1.22+/gRPC
- Priority: high
- Dependencies: 1

## Implementation Plan
1. Initialize Go module `github.com/5dlabs/sigma1-rms` with Go 1.22+.
2. Define protobuf files in `proto/sigma1/rms/v1/`:
   - `opportunity.proto`: CreateOpportunity, GetOpportunity, UpdateOpportunity, ListOpportunities, ScoreLead RPCs with full request/response messages per PRD
   - `project.proto`: CreateProject, GetProject, UpdateProject, CheckOut, CheckIn RPCs
   - `inventory.proto`: GetStockLevel, RecordTransaction, ScanBarcode RPCs
   - `crew.proto`: ListCrew, AssignCrew, ScheduleCrew RPCs
   - `delivery.proto`: ScheduleDelivery, UpdateDeliveryStatus, OptimizeRoute RPCs
   - Include grpc-gateway annotations for all RPCs mapping to REST endpoints per PRD
   - Include google.api.http options for each RPC
3. Generate Go code with `buf generate` (protoc-gen-go, protoc-gen-go-grpc, protoc-gen-grpc-gateway).
4. Database migrations (golang-migrate) in `rms` schema:
   - `opportunities` table: id, customer_id, status (enum: pending/qualified/approved/converted), event_date_start, event_date_end, venue, total_estimate_cents, lead_score (GREEN/YELLOW/RED), notes, created_at, updated_at
   - `opportunity_line_items` table: id, opportunity_id (FK), product_id, quantity, day_rate_cents, days, subtotal_cents
   - `projects` table: id, opportunity_id (FK), customer_id, status (confirmed/in_progress/completed/cancelled), confirmed_at, event_date_start, event_date_end, venue_address, crew_notes, created_at
   - `inventory_transactions` table: id, inventory_item_id, type (checkout/checkin/transfer), project_id (FK nullable), from_store_id, to_store_id, timestamp, user_id
   - `crew_members` table: id, name, role, phone, email, hourly_rate_cents
   - `crew_assignments` table: id, project_id (FK), crew_member_id (FK), role, start_time, end_time
   - `deliveries` table: id, project_id (FK), status, scheduled_at, vehicle_id, driver_id, pickup_address, delivery_address, notes
   - Indexes on foreign keys and status columns
5. Implement gRPC service handlers:
   - OpportunityService: full CRUD, ScoreLead computes GREEN/YELLOW/RED based on vetting data + opportunity value
   - ProjectService: CreateProject converts an approved opportunity, CheckOut/CheckIn record inventory transactions
   - InventoryService: stock level aggregation from transactions, barcode scan lookup
   - CrewService: assignment with conflict detection (double-booking check)
   - DeliveryService: schedule with basic route info (v1: no optimization, store address fields)
6. Implement grpc-gateway HTTP server on separate port (8081 for gRPC, 8080 for REST).
7. Middleware:
   - RBAC validation reading `sigma1-rbac-roles` ConfigMap JSON, validating JWT service tokens in Authorization header
   - Request logging with structured JSON (zerolog)
   - Prometheus metrics via `grpc_prometheus` interceptors + custom HTTP metrics
   - Health checks: gRPC health service + HTTP /health/live and /health/ready
8. Conflict detection for crew scheduling: query overlapping assignments before INSERT, return error if conflict.
9. Google Calendar integration (optional, behind feature flag): on project creation, create calendar event via Google Calendar API.
10. Kubernetes Deployment manifest:
    - Namespace: `sigma1`, 2 replicas
    - `envFrom: configMapRef: sigma1-infra-endpoints`
    - Ports: 8080 (REST), 8081 (gRPC)
    - Resource limits: 256Mi memory, 250m CPU
11. Dockerfile: multi-stage (golang:1.22-alpine builder, distroless runtime).
12. Connection pooling via pgxpool for PostgreSQL, go-redis/v9 for Valkey.

## Acceptance Criteria
1. Unit test: ScoreLead logic returns GREEN for customer_id with verified vetting + opportunity > $5000, YELLOW for partial vetting, RED for no vetting + high value (>= 3 scenarios). 2. Integration test: CreateOpportunity → UpdateOpportunity(status=approved) → CreateProject → CheckOut → CheckIn full lifecycle, verify all state transitions and inventory transaction records. 3. Integration test: crew scheduling conflict detection — assign crew member to overlapping time ranges, verify error returned on second assignment. 4. gRPC reflection test: `grpcurl -plaintext localhost:8081 list` returns all 5 service names. 5. grpc-gateway test: `curl localhost:8080/api/v1/opportunities` returns valid JSON with proper field name mapping (snake_case). 6. Prometheus metrics test: `/metrics` endpoint includes `grpc_server_handled_total` counter. 7. Health check test: `/health/ready` returns 200 when DB connected, 503 when connection pool exhausted. 8. Barcode scan test: RecordTransaction with barcode lookup resolves to correct inventory_item_id.

## Subtasks
- Initialize Go module and configure buf for protobuf code generation: Set up the Go module `github.com/5dlabs/sigma1-rms` with Go 1.22+, configure buf.yaml and buf.gen.yaml for protoc-gen-go, protoc-gen-go-grpc, and protoc-gen-grpc-gateway code generation. Establish the project directory structure including proto/, cmd/, internal/, migrations/, and deploy/ directories.
- Define OpportunityService and ProjectService protobuf schemas with grpc-gateway annotations: Author `opportunity.proto` and `project.proto` in `proto/sigma1/rms/v1/` with all RPCs, request/response messages, enums, and google.api.http annotations for REST mapping.
- Define InventoryService, CrewService, and DeliveryService protobuf schemas with grpc-gateway annotations: Author `inventory.proto`, `crew.proto`, and `delivery.proto` in `proto/sigma1/rms/v1/` with all RPCs, messages, and REST annotations.
- Create database migrations for all 7 RMS schema tables: Implement golang-migrate migration files for the `rms` schema including all 7 tables (opportunities, opportunity_line_items, projects, inventory_transactions, crew_members, crew_assignments, deliveries) with proper indexes, foreign keys, enums, and constraints.
- Implement database access layer with pgxpool connection management: Create the database access layer using pgxpool for connection pooling, including repository interfaces and implementations for all 7 tables with standard CRUD operations and query helpers.
- Implement OpportunityService gRPC handlers with ScoreLead logic: Implement the OpportunityService gRPC server with full CRUD operations for opportunities and line items, plus the ScoreLead RPC that computes GREEN/YELLOW/RED based on vetting data and opportunity value.
- Implement ProjectService gRPC handlers with opportunity conversion and inventory CheckOut/CheckIn: Implement the ProjectService gRPC server including project creation from approved opportunities, full CRUD, and CheckOut/CheckIn RPCs that record inventory transactions.
- Implement InventoryService gRPC handlers with stock aggregation and barcode scan: Implement the InventoryService gRPC server including stock level computation from transaction history and barcode-based item lookup.
- Implement CrewService gRPC handlers with scheduling conflict detection: Implement the CrewService gRPC server including crew listing, assignment creation with overlap conflict detection, and bulk scheduling.
- Implement DeliveryService gRPC handlers: Implement the DeliveryService gRPC server with delivery scheduling, status updates, and a stub OptimizeRoute endpoint for v1.
- Configure grpc-gateway HTTP server with REST endpoint mapping: Set up the grpc-gateway reverse proxy HTTP server on port 8080 that translates REST calls to gRPC calls on port 8081, with proper JSON serialization options.
- Implement RBAC validation and JWT authentication middleware: Create gRPC interceptors for JWT token validation from Authorization headers and RBAC role checking against the sigma1-rbac-roles ConfigMap.
- Implement structured logging, Prometheus metrics, and health check endpoints: Add zerolog structured JSON logging interceptor, grpc_prometheus metrics interceptors, custom HTTP metrics, and gRPC+HTTP health check endpoints.
- Create Dockerfile and Kubernetes deployment manifests: Create multi-stage Dockerfile for the RMS service and Kubernetes Deployment, Service, and ConfigMap reference manifests for the sigma1 namespace.
- Implement Valkey (Redis) integration for caching and session support: Add go-redis/v9 client initialization for Valkey connection pooling, reading connection details from the sigma1-infra-endpoints ConfigMap.
- Write end-to-end integration tests for full RMS service lifecycle: Create comprehensive integration tests covering the full opportunity-to-project lifecycle, crew conflict detection, inventory tracking, and both gRPC and REST interfaces.

## Deliverables
- Update the relevant code, configuration, and tests.
- Keep artifacts aligned with the acceptance criteria.
- Document blockers or assumptions in your final summary.