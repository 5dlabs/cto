Implement subtask 3007: Implement CrewService and DeliveryService

## Objective
Build the CrewService and DeliveryService gRPC handlers for crew assignment, availability management, delivery scheduling, and status tracking.

## Steps
1. Implement CrewService gRPC server in /internal/crew/. Wire up ListCrew, AssignCrew, GetAvailability, UpdateAvailability RPCs. AssignCrew should validate crew member availability for the given date range before creating assignment. GetAvailability should check existing assignments to compute open slots. 2. Implement DeliveryService gRPC server in /internal/delivery/. Wire up ScheduleDelivery, UpdateDeliveryStatus, ListDeliveries RPCs. ScheduleDelivery should validate equipment availability and create a delivery record with pickup/dropoff details. UpdateDeliveryStatus supports transitions: scheduled → in_transit → delivered → picked_up. 3. Register both services with gRPC server and verify grpc-gateway routes.

## Validation
Crew assignment respects availability constraints; double-booking a crew member for overlapping dates is rejected; delivery scheduling creates valid records; delivery status transitions follow the defined state machine; REST endpoints mirror gRPC behavior.