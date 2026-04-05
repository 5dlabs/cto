Implement subtask 3010: End-to-end workflow validation tests

## Objective
Write comprehensive end-to-end tests covering the full quote-to-project lifecycle, inventory check-in/check-out workflows, and cross-service interactions.

## Steps
1. Create `e2e/` test directory with Go test files using a test database.
2. Test 1 — Full Quote-to-Project lifecycle: create opportunity with line items → send quote → accept quote → convert to project → verify project has correct dates and items → assign crew → schedule delivery → transition delivery to delivered → check out inventory items → check in inventory items → verify complete audit trail.
3. Test 2 — Conflict detection: create two projects with overlapping dates → assign same crew member to both → verify conflict on second assignment → assign same inventory item to both → verify conflict detection.
4. Test 3 — Concurrent operations: simulate concurrent check-out of the same inventory item from two goroutines → verify exactly one succeeds.
5. Test 4 — REST parity: for each gRPC call in tests 1-3, make the equivalent REST call via grpc-gateway and verify responses match.
6. All tests should use testcontainers or a dedicated test database with transaction rollback.

## Validation
All e2e tests pass with `go test ./e2e/...`. Tests cover the complete happy path lifecycle, conflict scenarios, concurrency edge cases, and REST/gRPC parity. Test output shows no data leakage between test cases.