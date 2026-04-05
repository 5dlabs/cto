Implement subtask 3002: Define OpportunityService and ProjectService protobuf schemas with grpc-gateway annotations

## Objective
Author `opportunity.proto` and `project.proto` in `proto/sigma1/rms/v1/` with all RPCs, request/response messages, enums, and google.api.http annotations for REST mapping.

## Steps
1. Create `opportunity.proto`:
   - Package `sigma1.rms.v1`
   - Enums: OpportunityStatus (PENDING, QUALIFIED, APPROVED, CONVERTED), LeadScore (GREEN, YELLOW, RED)
   - Messages: Opportunity (all fields from DB schema including nested OpportunityLineItem repeated field), CreateOpportunityRequest, CreateOpportunityResponse, GetOpportunityRequest, GetOpportunityResponse, UpdateOpportunityRequest, UpdateOpportunityResponse, ListOpportunitiesRequest (with pagination: page_size, page_token, filter by status), ListOpportunitiesResponse, ScoreLeadRequest, ScoreLeadResponse
   - RPCs with google.api.http: CreateOpportunity (POST /api/v1/opportunities), GetOpportunity (GET /api/v1/opportunities/{id}), UpdateOpportunity (PATCH /api/v1/opportunities/{id}), ListOpportunities (GET /api/v1/opportunities), ScoreLead (POST /api/v1/opportunities/{id}/score)
2. Create `project.proto`:
   - Enums: ProjectStatus (CONFIRMED, IN_PROGRESS, COMPLETED, CANCELLED)
   - Messages: Project (all DB fields), CreateProjectRequest (from opportunity_id), CreateProjectResponse, GetProjectRequest, GetProjectResponse, UpdateProjectRequest, UpdateProjectResponse, CheckOutRequest (project_id, inventory_item_id, quantity), CheckOutResponse, CheckInRequest, CheckInResponse
   - RPCs: CreateProject (POST /api/v1/projects), GetProject (GET /api/v1/projects/{id}), UpdateProject (PATCH /api/v1/projects/{id}), CheckOut (POST /api/v1/projects/{id}/checkout), CheckIn (POST /api/v1/projects/{id}/checkin)
3. Use google.protobuf.Timestamp for all date/time fields.
4. Run `buf lint` and fix any issues.
5. Run `buf generate` and verify Go code compiles.

## Validation
Run `buf lint` with zero errors. Run `buf generate` and verify generated Go files compile with `go build ./...`. Verify HTTP annotations exist by inspecting generated `.pb.gw.go` files for route registrations.