Implement subtask 3008: Implement booking conflict detection across all services

## Objective
Build a cross-cutting conflict detection system that checks for scheduling conflicts across projects, crew assignments, inventory availability, and delivery windows before confirming bookings.

## Steps
1. Create `internal/conflicts/` package with a ConflictChecker service.
2. Implement resource conflict detection: given a project with date range, check (a) all required inventory items are available for those dates, (b) all assigned crew members are available, (c) delivery windows don't overlap with other deliveries for the same address/vehicle.
3. Expose a CheckConflicts RPC that takes a project_id or proposed booking and returns a list of all conflicts grouped by type (inventory, crew, delivery).
4. Integrate conflict checking into the ProjectService.CreateProject and ProjectService.UpdateProject flows as a pre-validation step (warn or block based on configuration).
5. Add an `internal/audit/` package: create an audit_log table, write audit entries for all state-changing operations across services (opportunity status changes, project creation, check-in/check-out, crew assignments, delivery status changes). Each entry records: entity_type, entity_id, action, actor, timestamp, old_value, new_value as JSON.

## Validation
Integration test: create a project with crew and inventory, then attempt to create an overlapping project using the same resources — verify conflicts are detected and returned with correct details. Verify audit log entries are created for all major operations across services. Query audit log by entity_type and entity_id to verify completeness.