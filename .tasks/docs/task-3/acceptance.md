## Acceptance Criteria

- [ ] 1. Unit test: ScoreLead logic returns GREEN for customer_id with verified vetting + opportunity > $5000, YELLOW for partial vetting, RED for no vetting + high value (>= 3 scenarios). 2. Integration test: CreateOpportunity → UpdateOpportunity(status=approved) → CreateProject → CheckOut → CheckIn full lifecycle, verify all state transitions and inventory transaction records. 3. Integration test: crew scheduling conflict detection — assign crew member to overlapping time ranges, verify error returned on second assignment. 4. gRPC reflection test: `grpcurl -plaintext localhost:8081 list` returns all 5 service names. 5. grpc-gateway test: `curl localhost:8080/api/v1/opportunities` returns valid JSON with proper field name mapping (snake_case). 6. Prometheus metrics test: `/metrics` endpoint includes `grpc_server_handled_total` counter. 7. Health check test: `/health/ready` returns 200 when DB connected, 503 when connection pool exhausted. 8. Barcode scan test: RecordTransaction with barcode lookup resolves to correct inventory_item_id.

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.