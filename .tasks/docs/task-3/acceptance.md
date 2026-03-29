## Acceptance Criteria

- [ ] 1. Deploy the service to Kubernetes and verify it starts successfully.2. Use `grpcurl` to test gRPC endpoints for `CreateOpportunity`, `GetProject`, `RecordTransaction`, etc.3. Use `curl` or Postman to verify REST endpoints exposed via `grpc-gateway` are functional.4. Test the full quote-to-project workflow: create opportunity, approve, convert to project.5. Verify inventory transactions can be recorded and retrieved.6. Confirm data persistence in PostgreSQL for all RMS entities.

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.