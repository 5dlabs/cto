Implement task 3: Build Rental Management System Service (Grizz - Go/gRPC)

## Goal
Implement the full Rental Management System (RMS) as a Go gRPC service with grpc-gateway REST bridge. Handles opportunities (quotes), projects, inventory transactions, crew management, and delivery scheduling. This is the operational backbone replacing Current RMS. Exposes both native gRPC and REST endpoints. All other services communicate via the REST gateway. Includes Google Calendar integration and internal GDPR endpoints.

## Task Context
- Agent owner: grizz
- Stack: Go 1.22+/gRPC/grpc-gateway
- Priority: high
- Dependencies: 1

## Implementation Plan
1. Initialize Go module at services/rms. Dependencies (go.mod): google.golang.org/grpc v1.63+, google.golang.org/protobuf v1.33+, github.com/grpc-ecosystem/grpc-gateway/v2 v2.19+, github.com/jackc/pgx/v5, github.com/redis/go-redis/v9, github.com/google/uuid, github.com/shopspring/decimal, go.uber.org/zap, github.com/prometheus/client_golang, google.golang.org/api (calendar), github.com/golang-migrate/migrate/v4.
2. Define protobuf files in proto/sigma1/rms/v1/: opportunity.proto, project.proto, inventory.proto, crew.proto, delivery.proto. Generate with buf generate. Include grpc-gateway annotations for REST mapping.
3. Database migrations in migrations/ targeting rms schema (SET search_path=rms). Tables: opportunities (id UUID PK, customer_id UUID, status TEXT CHECK IN (pending,qualified,approved,converted), event_date_start TIMESTAMPTZ, event_date_end TIMESTAMPTZ, venue TEXT, total_estimate NUMERIC(12,2), lead_score TEXT CHECK IN (GREEN,YELLOW,RED), notes TEXT, created_at TIMESTAMPTZ, updated_at TIMESTAMPTZ), opportunity_line_items (id UUID PK, opportunity_id UUID FK, product_id UUID, quantity INT, day_rate NUMERIC(10,2)), projects (id UUID PK, opportunity_id UUID FK, customer_id UUID, status TEXT, confirmed_at TIMESTAMPTZ, event_date_start TIMESTAMPTZ, event_date_end TIMESTAMPTZ, venue_address TEXT, crew_notes TEXT), inventory_transactions (id UUID PK, inventory_id UUID, type TEXT CHECK IN (checkout,checkin,transfer), project_id UUID, from_store_id UUID, to_store_id UUID, timestamp TIMESTAMPTZ, user_id UUID), crew_members (id UUID PK, name TEXT, email TEXT, phone TEXT, role TEXT), crew_assignments (id UUID PK, project_id UUID FK, crew_member_id UUID FK, role TEXT, notes TEXT), deliveries (id UUID PK, project_id UUID FK, status TEXT, scheduled_at TIMESTAMPTZ, completed_at TIMESTAMPTZ, route_data JSONB).
4. Implement gRPC service handlers for OpportunityService (CreateOpportunity, GetOpportunity, UpdateOpportunity, ListOpportunities, ScoreLead), ProjectService (CreateProject, GetProject, UpdateProject, CheckOut, CheckIn), InventoryService (GetStockLevel, RecordTransaction, ScanBarcode), CrewService (ListCrew, AssignCrew, ScheduleCrew), DeliveryService (ScheduleDelivery, UpdateDeliveryStatus, OptimizeRoute).
5. REST gateway: run grpc-gateway mux on :8080, gRPC server on :9090. All external consumers and Morgan use :8080.
6. ScoreLead: weighted algorithm on opportunity data: event size, venue history, customer vetting score (fetched from vetting service via HTTP GET /api/v1/vetting/:org_id), lead age. Returns GREEN/YELLOW/RED.
7. Google Calendar integration: on Project confirmed status, create Calendar event via google.golang.org/api/calendar/v3 using GOOGLE_CALENDAR_CLIENT_ID/SECRET from sigma1-google-secret. Store event ID in project row.
8. Conflict detection: on checkout, verify inventory_transactions for same inventory_id in overlapping date window; return ConflictError if already checked out.
9. Prometheus metrics and /health/live, /health/ready endpoints on :8081 (separate HTTP mux to avoid grpc-gateway conflicts).
10. GDPR: GET /internal/gdpr/export/:customer_id returns all opportunities, projects, transactions for customer. DELETE /internal/gdpr/delete/:customer_id anonymizes customer_id fields (replace with NULL and log to audit schema).
11. Kubernetes Deployment: 2 replicas, envFrom sigma1-infra-endpoints + secrets, probes on :8081, gRPC port 9090 internal only, REST port 8080 exposed via ClusterIP Service.
12. Unit tests for ScoreLead algorithm and conflict detection. Integration tests using pgx test containers.

## Acceptance Criteria
1. POST /api/v1/opportunities with valid payload returns 201 with id field and status=pending. 2. POST /api/v1/opportunities/:id/convert returns 200 and creates a project row (verify via GET /api/v1/projects/:id). 3. ScoreLead returns GREEN for opportunity with verified customer, YELLOW for unvetted, RED for flagged — verified by seeding vetting results and calling POST /api/v1/opportunities/:id/score. 4. CheckOut creates inventory_transaction of type=checkout; second CheckOut for same item in same window returns 409 Conflict. 5. GET /health/ready returns 200 with postgres and redis reachable. 6. gRPC reflection accessible on port 9090 (grpcurl list returns service names). 7. GET /internal/gdpr/delete/:id anonymizes customer_id in opportunities and projects (re-query returns null customer_id). 8. go test ./... passes with >= 80% coverage.

## Subtasks
- Initialize Go module and dependency manifest for RMS service: Create the Go module at services/rms with go.mod declaring all required dependencies: grpc, protobuf, grpc-gateway, pgx/v5, go-redis, uuid, decimal, zap, prometheus, google.golang.org/api, golang-migrate.
- Define protobuf schemas for all five RMS domains: Write proto/sigma1/rms/v1/opportunity.proto, project.proto, inventory.proto, crew.proto, and delivery.proto with grpc-gateway HTTP annotations for REST mapping. Run buf generate to produce Go stubs.
- Write database migrations for all seven RMS schema tables: Create numbered SQL migration files in services/rms/migrations/ targeting the rms schema. Cover all seven tables: opportunities, opportunity_line_items, projects, inventory_transactions, crew_members, crew_assignments, deliveries.
- Implement OpportunityService gRPC handlers with REST gateway: Implement all five OpportunityService methods (CreateOpportunity, GetOpportunity, UpdateOpportunity, ListOpportunities, ScoreLead stub) as gRPC handler structs backed by pgx queries. Wire into grpc-gateway mux on :8080 and gRPC server on :9090.
- Implement ProjectService gRPC handlers including CheckOut and CheckIn: Implement all five ProjectService methods (CreateProject, GetProject, UpdateProject, CheckOut, CheckIn) as gRPC handlers. CheckOut must write an inventory_transaction row; CheckIn must record a checkin transaction.
- Implement InventoryService with atomic conflict detection and unit tests: Implement InventoryService handlers (GetStockLevel, RecordTransaction, ScanBarcode) and the inventory conflict detection logic that returns ConflictError on overlapping checkout windows. Write unit tests for conflict detection.
- Implement CrewService and DeliveryService gRPC handlers: Implement CrewService (ListCrew, AssignCrew, ScheduleCrew) and DeliveryService (ScheduleDelivery, UpdateDeliveryStatus, OptimizeRoute) as gRPC handlers backed by pgx.
- Implement ScoreLead algorithm with external vetting service HTTP call and unit tests: Replace the ScoreLead stub with the full weighted scoring algorithm that calls the vetting service HTTP endpoint, computes a composite score from event size, venue history, customer vetting score, and lead age, and returns GREEN/YELLOW/RED.
- Implement Google Calendar integration triggered on project confirmation: On ProjectService UpdateProject when status transitions to confirmed, create a Google Calendar event via the Calendar v3 API and store the returned event ID in the project row.
- Add Prometheus metrics and health endpoints on dedicated port 8081: Expose /metrics, /health/live, and /health/ready endpoints on a separate net/http mux listening on :8081 to avoid conflicts with the grpc-gateway mux on :8080.
- Implement GDPR export and delete endpoints: Implement GET /internal/gdpr/export/:customer_id and DELETE /internal/gdpr/delete/:customer_id on the grpc-gateway mux. Export returns all customer data; delete anonymizes customer_id fields and logs to audit schema.
- Write Kubernetes Deployment and Service manifests for RMS: Create Kubernetes Deployment (2 replicas) and ClusterIP Service manifests for the RMS service with correct port configuration, envFrom references, and health probes.
- Write integration tests for RMS service end-to-end flows: Write integration tests using pgx testcontainers covering the full opportunity-to-project flow, inventory conflict detection producing 409, and GDPR anonymization correctness.

## Deliverables
- Update the relevant code, configuration, and tests.
- Keep artifacts aligned with the acceptance criteria.
- Document blockers or assumptions in your final summary.