Implement subtask 3006: Define protobuf schemas for CrewService and DeliveryService

## Objective
Write .proto files for CrewService (crew management and scheduling) and DeliveryService (delivery/pickup tracking) with grpc-gateway annotations.

## Steps
1. Create `proto/rms/v1/crew.proto`:
   - Messages: CrewMember, CrewAssignment, CreateCrewMemberRequest/Response, GetCrewMemberRequest/Response, ListCrewMembersRequest/Response, AssignCrewRequest/Response, GetCrewAvailabilityRequest/Response, UnassignCrewRequest/Response
   - RPCs: CreateCrewMember, GetCrewMember, ListCrewMembers, AssignCrewToProject, UnassignCrewFromProject, GetCrewAvailability
   - HTTP annotations for REST endpoints.
2. Create `proto/rms/v1/delivery.proto`:
   - Messages: Delivery, CreateDeliveryRequest/Response, GetDeliveryRequest/Response, ListDeliveriesRequest/Response, UpdateDeliveryStatusRequest/Response
   - Enums: DeliveryType (DELIVERY, PICKUP), DeliveryStatus (SCHEDULED, IN_TRANSIT, COMPLETED)
   - RPCs: CreateDelivery, GetDelivery, ListDeliveries, UpdateDeliveryStatus
   - HTTP annotations for REST endpoints.
3. Run `make proto-gen` and verify all generated Go code compiles.

## Validation
Both proto files compile without errors. Generated Go code compiles. All RPCs have correct HTTP annotations. CrewAvailability response includes date-range availability information.