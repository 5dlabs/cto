Implement subtask 3002: Define opportunity.proto and project.proto with grpc-gateway annotations

## Objective
Create protobuf definitions for the Opportunity and Project gRPC services with full REST gateway annotations for all endpoints including approve and convert actions.

## Steps
1. Create `proto/rms/v1/opportunity.proto`:
   - Service `OpportunityService` with RPCs: `CreateOpportunity`, `GetOpportunity`, `UpdateOpportunity`, `ListOpportunities`, `ScoreLead`, `ApproveOpportunity`, `ConvertOpportunity`.
   - Messages: `Opportunity` (id, org_id, customer_id, title, description, event_date_start, event_date_end, status enum [PENDING, QUALIFIED, APPROVED, CONVERTED], lead_score, line_items repeated, created_at, updated_at), `LeadScore` (score enum [GREEN, YELLOW, RED], breakdown map), `OpportunityLineItem`.
   - grpc-gateway annotations: `POST /api/v1/opportunities`, `GET /api/v1/opportunities/{id}`, `PUT /api/v1/opportunities/{id}`, `GET /api/v1/opportunities`, `POST /api/v1/opportunities/{id}/score`, `POST /api/v1/opportunities/{id}/approve`, `POST /api/v1/opportunities/{id}/convert`.
2. Create `proto/rms/v1/project.proto`:
   - Service `ProjectService` with RPCs: `CreateProject`, `GetProject`, `UpdateProject`, `ListProjects`, `CheckOut`, `CheckIn`.
   - Messages: `Project` (id, org_id, opportunity_id, customer_id, title, status, line_items, checkout_date, checkin_date, created_at, updated_at), `CheckOutRequest` (project_id, item_ids, date_range), `CheckOutResponse` (success, conflicts repeated), `Conflict` (item_id, conflicting_project_id, date_range).
   - grpc-gateway annotations: `POST /api/v1/projects`, `GET /api/v1/projects/{id}`, `PUT /api/v1/projects/{id}`, `GET /api/v1/projects`, `POST /api/v1/projects/{id}/checkout`, `POST /api/v1/projects/{id}/checkin`.
3. Run `buf generate` and verify Go stubs compile.

## Validation
Run `buf lint` with zero errors on both proto files. Run `buf generate` and verify generated Go service interfaces contain all defined RPCs. Verify grpc-gateway reverse proxy code is generated with correct HTTP method/path mappings.