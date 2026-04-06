Implement subtask 3008: Implement CrewService and DeliveryService gRPC handlers with conflict detection

## Objective
Implement CrewService and DeliveryService handlers including crew assignment conflict detection that prevents double-booking crew members for overlapping date ranges.

## Steps
1. Create internal/service/crew_service.go implementing CrewServiceServer.
2. Implement CRUD RPCs for crew members.
3. Implement AssignToProject RPC with conflict detection:
   - Before assigning, query existing assignments for the crew member that overlap the requested date range
   - Use PostgreSQL range overlap check (daterange && daterange) or exclusion constraint
   - If conflict found, return FailedPrecondition with details of conflicting assignment
   - On success, create assignment record
4. Implement UnassignFromProject RPC.
5. Implement CheckAvailability RPC that returns crew members available for a given date range and optional skill filter.
6. Create internal/service/delivery_service.go implementing DeliveryServiceServer.
7. Implement CRUD RPCs for deliveries.
8. Implement UpdateDeliveryStatus with valid state transitions (scheduled → in_transit → delivered, or scheduled → cancelled).
9. Implement AssignDriver RPC with conflict detection (driver can't have overlapping deliveries).
10. Register both services with gRPC server.

## Validation
AssignToProject with non-overlapping dates succeeds; AssignToProject with overlapping dates for same crew member returns conflict error; CheckAvailability excludes already-assigned crew for the requested range; UpdateDeliveryStatus enforces valid state transitions; AssignDriver detects scheduling conflicts; all RPCs return appropriate gRPC status codes.