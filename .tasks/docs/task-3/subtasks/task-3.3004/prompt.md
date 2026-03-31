Implement subtask 3004: Define protobuf schemas for OpportunityService and ProjectService

## Objective
Write .proto files for OpportunityService and ProjectService including all message types, enums, RPC methods, and grpc-gateway HTTP annotations for the quote-to-project workflow.

## Steps
1. Create `proto/rms/v1/opportunity.proto`:
   - Messages: Opportunity, CreateOpportunityRequest/Response, GetOpportunityRequest/Response, ListOpportunitiesRequest/Response, UpdateOpportunityRequest/Response, ConvertToProjectRequest/Response
   - Enums: OpportunityStatus (LEAD, QUOTED, WON, LOST)
   - RPCs: CreateOpportunity, GetOpportunity, ListOpportunities, UpdateOpportunity, ConvertOpportunityToProject
   - Add `google.api.http` annotations for REST mapping (POST /api/v1/opportunities, GET /api/v1/opportunities/{id}, etc.)
2. Create `proto/rms/v1/project.proto`:
   - Messages: Project, CreateProjectRequest/Response, GetProjectRequest/Response, ListProjectsRequest/Response, UpdateProjectRequest/Response
   - Enums: ProjectStatus (PENDING, ACTIVE, COMPLETED, CANCELLED)
   - RPCs: CreateProject, GetProject, ListProjects, UpdateProject
   - Add grpc-gateway HTTP annotations.
3. Create shared `proto/rms/v1/common.proto` for pagination, timestamps, and shared field types.
4. Run `make proto-gen` and verify generated Go code compiles.

## Validation
Proto files compile without errors via `buf lint` and `buf generate`. Generated Go code compiles. HTTP annotations are present and correctly map to REST paths. All message fields match the database schema.