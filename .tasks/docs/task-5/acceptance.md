## Acceptance Criteria

- [ ] 1. Use `grpcurl` and `curl` to test all new gRPC and REST endpoints for Inventory, Crew, and Delivery services. 2. Verify that inventory transactions are correctly recorded and stock levels are updated. 3. Confirm crew members can be listed and assigned to projects. 4. Verify delivery schedules can be created and updated. 5. Check that the Google Calendar API stub logs expected calls. 6. Run `staticcheck ./...` to identify potential issues in the Go codebase. 7. Ensure all new database models are correctly created and data integrity is maintained.

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.