Implement subtask 3007: Implement CrewService and DeliveryService gRPC handlers

## Objective
Implement CrewService (ListCrew, AssignCrew, ScheduleCrew) and DeliveryService (ScheduleDelivery, UpdateDeliveryStatus, OptimizeRoute) as gRPC handlers backed by pgx.

## Steps
Create internal/crew/handler.go: ListCrew queries crew_members. AssignCrew inserts a crew_assignments row linking crew_member_id to project_id with role. ScheduleCrew updates crew_assignment notes/role. Create internal/delivery/handler.go: ScheduleDelivery inserts deliveries row with status=scheduled and scheduled_at. UpdateDeliveryStatus updates status and completed_at. OptimizeRoute accepts a list of delivery IDs and returns them in a stub-ordered sequence (sort by scheduled_at ascending); store order in route_data JSONB. Register both service servers on the shared gRPC instance. Wire grpc-gateway HTTP routes.

## Validation
POST /api/v1/crew/assign creates crew_assignments row retrievable by project_id. POST /api/v1/deliveries schedules a delivery. PATCH /api/v1/deliveries/:id/status updates status to completed. POST /api/v1/deliveries/optimize returns ordered array of delivery IDs.