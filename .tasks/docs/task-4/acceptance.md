## Acceptance Criteria

- [ ] 1. Deploy the service and verify it starts successfully, connecting to PostgreSQL. 2. Use `grpcurl` to test gRPC endpoints for creating, retrieving, updating, and listing opportunities and projects. 3. Use `curl` or Postman to test the REST endpoints exposed by `grpc-gateway` for the same operations. 4. Verify that data persisted via gRPC is correctly retrieved via REST and vice-versa. 5. Confirm `ScoreLead` returns a valid `LeadScore` (even if placeholder logic). 6. Run `go test ./...` and `go vet ./...` to ensure code quality and correctness. 7. Verify database schema matches protobuf definitions.

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.