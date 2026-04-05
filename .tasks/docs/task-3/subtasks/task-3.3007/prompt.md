Implement subtask 3007: Implement ProjectService gRPC handlers with opportunity conversion and inventory CheckOut/CheckIn

## Objective
Implement the ProjectService gRPC server including project creation from approved opportunities, full CRUD, and CheckOut/CheckIn RPCs that record inventory transactions.

## Steps
1. Create `internal/service/project.go` implementing ProjectServiceServer.
2. Implement CreateProject:
   - Accept opportunity_id in request
   - Verify opportunity exists and status is 'approved' (return FailedPrecondition if not)
   - Create project record copying relevant fields from opportunity (customer_id, event dates, venue)
   - Update opportunity status to 'converted' in a database transaction
   - Return created project
3. Implement GetProject: lookup by ID, return NOT_FOUND if missing.
4. Implement UpdateProject: support status transitions (confirmed→in_progress→completed; confirmed→cancelled; in_progress→cancelled). Validate transitions.
5. Implement CheckOut:
   - Accept project_id, inventory_item_id, quantity
   - Verify project exists and is in_progress
   - Check stock level >= requested quantity via InventoryRepo.GetStockLevel
   - Record inventory_transaction with type=checkout
   - Return updated stock level
6. Implement CheckIn:
   - Accept project_id, inventory_item_id, quantity
   - Record inventory_transaction with type=checkin
   - Return updated stock level
7. Both CheckOut and CheckIn should be wrapped in database transactions to ensure atomicity.
8. Register the service in the gRPC server.

## Validation
Integration test: full lifecycle CreateOpportunity → approve → CreateProject → verify opportunity status is 'converted'. Test CheckOut reduces stock level and CheckIn restores it. Test CheckOut with insufficient stock returns appropriate error. Test invalid state transitions return FailedPrecondition.