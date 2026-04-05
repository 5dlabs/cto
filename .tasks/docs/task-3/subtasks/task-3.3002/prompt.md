Implement subtask 3002: Define protobuf service definitions for all five RMS domains

## Objective
Create .proto files for OpportunityService, ProjectService, InventoryService, CrewService, and DeliveryService with all message types, RPC methods, and grpc-gateway HTTP annotations.

## Steps
1. Create `proto/rms/v1/` directory structure.
2. Define `opportunity.proto`: messages for Opportunity, QuoteLineItem, CreateOpportunityRequest/Response, GetOpportunityRequest/Response, ListOpportunitiesRequest/Response, UpdateOpportunityStatusRequest/Response, ConvertToProjectRequest/Response. RPCs: CreateOpportunity, GetOpportunity, ListOpportunities, UpdateOpportunityStatus, ConvertToProject.
3. Define `project.proto`: messages for Project, ProjectSchedule, CreateProjectRequest/Response, GetProjectRequest/Response, ListProjectsRequest/Response, UpdateProjectRequest/Response. RPCs: CreateProject, GetProject, ListProjects, UpdateProject.
4. Define `inventory.proto`: messages for InventoryItem, InventoryTransaction, CheckOutRequest/Response, CheckInRequest/Response, GetItemRequest/Response, ListItemsRequest/Response, ScanBarcodeRequest/Response. RPCs: CheckOutItem, CheckInItem, GetItem, ListItems, ScanBarcode.
5. Define `crew.proto`: messages for CrewMember, CrewAssignment, AssignCrewRequest/Response, GetCrewMemberRequest/Response, ListCrewRequest/Response, ListAssignmentsRequest/Response. RPCs: AssignCrew, UnassignCrew, GetCrewMember, ListCrew, ListAssignments.
6. Define `delivery.proto`: messages for Delivery, ScheduleDeliveryRequest/Response, UpdateDeliveryStatusRequest/Response, ListDeliveriesRequest/Response. RPCs: ScheduleDelivery, UpdateDeliveryStatus, ListDeliveries.
7. Add `google.api.http` annotations on all RPCs for grpc-gateway REST mapping.
8. Create `buf.gen.yaml` or Makefile target for `protoc` code generation.
9. Generate Go stubs and grpc-gateway reverse proxy code.

## Validation
All .proto files compile without errors via `protoc` or `buf build`. Generated Go code compiles. HTTP annotations are present and produce valid OpenAPI spec via grpc-gateway's swagger generation.