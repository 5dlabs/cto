Implement subtask 3002: Define protobuf schemas for all five RMS services and generate Go code

## Objective
Author .proto files for OpportunityService, ProjectService, InventoryService, CrewService, and DeliveryService with all message types, enums, and grpc-gateway HTTP annotations as per the PRD.

## Steps
1. Create proto/rms/v1/opportunity.proto: Define Opportunity message (id, customer_id, title, status enum [NEW, QUALIFIED, QUOTED, WON, LOST], notes, timestamps). Define OpportunityService with RPCs: CreateOpportunity, GetOpportunity, ListOpportunities, UpdateOpportunity, ConvertToProject. Add grpc-gateway google.api.http annotations for REST mappings.
2. Create proto/rms/v1/project.proto: Define Project message (id, opportunity_id, name, status enum [PLANNING, ACTIVE, ON_HOLD, COMPLETED], start_date, end_date, assigned_crew_ids, equipment_ids, delivery_ids). Define ProjectService RPCs: CreateProject, GetProject, ListProjects, UpdateProject, AssignCrew, AssignEquipment.
3. Create proto/rms/v1/inventory.proto: Define InventoryItem message (id, barcode, name, category, status enum [AVAILABLE, RESERVED, CHECKED_OUT, MAINTENANCE], location, last_scanned_at). Define InventoryService RPCs: CreateItem, GetItem, ListItems, ScanBarcode, UpdateItemStatus, CheckOut, CheckIn.
4. Create proto/rms/v1/crew.proto: Define CrewMember message (id, name, role, availability, calendar_event_id). Define CrewService RPCs: CreateMember, GetMember, ListMembers, UpdateMember, GetSchedule, AssignToProject.
5. Create proto/rms/v1/delivery.proto: Define Delivery message (id, project_id, equipment_ids, status enum [SCHEDULED, IN_TRANSIT, DELIVERED, RETURNED], pickup_time, delivery_time, driver_id, tracking_notes). Define DeliveryService RPCs: CreateDelivery, GetDelivery, ListDeliveries, UpdateDeliveryStatus, TrackDelivery.
6. Run buf generate or protoc to produce Go stubs and grpc-gateway reverse proxy code in /gen.
7. Verify generated code compiles cleanly.

## Validation
All five .proto files pass buf lint or protoc validation with zero errors; generated Go code compiles without errors; grpc-gateway HTTP annotations are present for every RPC.