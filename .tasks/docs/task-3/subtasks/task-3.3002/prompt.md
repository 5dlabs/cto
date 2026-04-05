Implement subtask 3002: Define protobuf schemas for all five RMS services

## Objective
Author .proto files for OpportunityService, ProjectService, InventoryService, CrewService, and DeliveryService with all message types, RPC methods, and grpc-gateway HTTP annotations.

## Steps
1. Create proto/opportunity.proto: messages for Opportunity (id, company, contact, equipment_needs, estimated_value, score, status), CreateOpportunityRequest/Response, ListOpportunitiesRequest/Response, UpdateOpportunityRequest. RPCs: CreateOpportunity, ListOpportunities, UpdateOpportunity, ScoreOpportunity, ConvertToProject. Add google.api.http annotations for REST mapping. 2. Create proto/project.proto: messages for Project (id, opportunity_id, status, schedule, crew_assignments, equipment_list, delivery_info), Quote message. RPCs: CreateProject, GetProject, ListProjects, UpdateProjectStatus, GenerateQuote, ApproveQuote. 3. Create proto/inventory.proto: messages for InventoryItem (id, barcode, name, category, status, location, condition), InventoryTransaction. RPCs: ScanBarcode, CheckOut, CheckIn, ListInventory, GetItem, RecordTransaction. 4. Create proto/crew.proto: messages for CrewMember, CrewAssignment, Availability. RPCs: ListCrew, AssignCrew, GetAvailability, UpdateAvailability. 5. Create proto/delivery.proto: messages for Delivery, DeliverySchedule. RPCs: ScheduleDelivery, UpdateDeliveryStatus, ListDeliveries. 6. Run protoc to generate Go stubs and gateway code. Verify all generated files compile.

## Validation
All .proto files pass `protoc --lint`; generated Go code compiles without errors; grpc-gateway annotations produce valid HTTP route mappings verified by inspecting the generated .pb.gw.go files.