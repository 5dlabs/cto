Implement subtask 3004: Define protobuf schemas for CrewService and DeliveryService

## Objective
Author .proto files for CrewService (crew member management, assignments, availability) and DeliveryService (delivery scheduling, tracking, status updates).

## Steps
1. Create proto/rms/v1/crew.proto with:
   - CrewMember message (id, name, role, skills, availability_schedule, contact_info)
   - Assignment message (crew_member_id, project_id, date_range, role)
   - RPCs: CreateCrewMember, GetCrewMember, ListCrewMembers, UpdateCrewMember, AssignToProject, UnassignFromProject, CheckAvailability
   - HTTP annotations
2. Create proto/rms/v1/delivery.proto with:
   - Delivery message (id, project_id, items, origin, destination, scheduled_at, status, driver_id, tracking_notes)
   - RPCs: CreateDelivery, GetDelivery, ListDeliveries, UpdateDeliveryStatus, AssignDriver
   - HTTP annotations
3. Run code generation and verify compilation.
4. Register both services in grpc-gateway mux.

## Validation
Generated Go code compiles; CrewService and DeliveryService RPCs match PRD specifications; HTTP annotations produce correct REST routes; all message fields are present.