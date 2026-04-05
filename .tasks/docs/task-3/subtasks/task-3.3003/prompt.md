Implement subtask 3003: Define protobuf schemas for all five RMS services

## Objective
Write .proto files for OpportunityService, ProjectService, InventoryService, CrewService, and DeliveryService, including all request/response messages, enums, and grpc-gateway HTTP annotations.

## Steps
1. Create proto/rms/v1/opportunity.proto: Define OpportunityService with RPCs: CreateOpportunity, GetOpportunity, ListOpportunities, UpdateOpportunity, ConvertToProject. Include message types for Opportunity, CreateOpportunityRequest/Response, etc. Add google.api.http annotations for REST mapping (e.g., POST /api/v1/opportunities). 2. Create proto/rms/v1/project.proto: Define ProjectService with RPCs: GetProject, ListProjects, UpdateProject, AssignInventory, AssignCrew, GetProjectTimeline. 3. Create proto/rms/v1/inventory.proto: Define InventoryService with RPCs: CreateItem, GetItem, ListItems, UpdateItem, ScanBarcode (takes barcode string, returns item), CheckoutItems, ReturnItems. 4. Create proto/rms/v1/crew.proto: Define CrewService with RPCs: CreateCrewMember, GetCrewMember, ListCrewMembers, UpdateCrewMember, GetAvailability, ScheduleCrewMember. 5. Create proto/rms/v1/delivery.proto: Define DeliveryService with RPCs: CreateDelivery, GetDelivery, ListDeliveries, UpdateDeliveryStatus, TrackDelivery. 6. Use shared proto for common types (Timestamp, Pagination, Money). 7. Ensure all RPCs have grpc-gateway HTTP annotations.

## Validation
All .proto files pass `protoc --lint` or buf lint without errors; grpc-gateway annotations are present on every RPC; message types cover all fields from the database schema.