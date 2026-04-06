Implement subtask 3005: Implement ProjectService gRPC handlers including CheckOut and CheckIn

## Objective
Implement all five ProjectService methods (CreateProject, GetProject, UpdateProject, CheckOut, CheckIn) as gRPC handlers. CheckOut must write an inventory_transaction row; CheckIn must record a checkin transaction.

## Steps
Create internal/project/handler.go. CreateProject: INSERT projects row, set status=active, return created. GetProject: SELECT by id. UpdateProject: UPDATE fields including confirmed_at when status transitions to confirmed. CheckOut: within a single pgx transaction, INSERT inventory_transaction with type=checkout; conflict detection is handled by the InventoryService (subtask 3006) — CheckOut in ProjectService calls internal conflict check before inserting. CheckIn: INSERT inventory_transaction with type=checkin. Register ProjectServiceServer on the same gRPC server as OpportunityService. Wire grpc-gateway routes.

## Validation
POST /api/v1/projects creates a project linked to an opportunity. POST /api/v1/projects/:id/checkout creates an inventory_transaction row of type=checkout. POST /api/v1/projects/:id/checkin creates a checkin row. GET /api/v1/projects/:id reflects updated status after UpdateProject.