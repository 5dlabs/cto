Implement subtask 3002: Define protobuf schemas for OpportunityService and ProjectService

## Objective
Author .proto files for OpportunityService and ProjectService with all RPC methods, request/response messages, and grpc-gateway HTTP annotations as specified in the PRD.

## Steps
1. Create proto/rms/v1/opportunity.proto with:
   - Opportunity message (id, client info, project name, status, dates, notes, equipment list)
   - CreateOpportunity, GetOpportunity, ListOpportunities, UpdateOpportunity, DeleteOpportunity RPCs
   - google.api.http annotations for REST mapping (POST /v1/opportunities, GET /v1/opportunities/{id}, etc.)
2. Create proto/rms/v1/project.proto with:
   - Project message (id, opportunity_id, name, status, schedule, crew assignments, calendar_event_id)
   - CreateProject, GetProject, ListProjects, UpdateProject, DeleteProject, SyncCalendar RPCs
   - HTTP annotations for grpc-gateway
3. Define shared messages in proto/rms/v1/common.proto (pagination, timestamps, status enums).
4. Run buf generate or protoc to generate Go stubs and grpc-gateway code.
5. Verify generated code compiles and integrates with the gRPC server scaffold.

## Validation
protoc/buf generates Go code without errors; generated stubs compile; RPC method signatures match PRD requirements; HTTP annotations produce correct REST routes when registered with grpc-gateway.