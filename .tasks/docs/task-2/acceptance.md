## Acceptance Criteria

- [ ] 1. Unit tests for availability calculation logic: given reservations and bookings, verify quantity_available is correct (>= 5 test cases). 2. Integration test: POST a product as admin, GET it back, verify all fields match including image CDN URLs. 3. Integration test: check availability for a product with existing bookings returns correct available quantity within < 500ms (measure with `std::time::Instant`). 4. Rate limiting test: send 101 requests in rapid succession from same IP, verify 429 status on request 101. 5. RBAC test: attempt POST /api/v1/catalog/products without admin role, verify 403. 6. Machine-readable endpoint test: GET /api/v1/equipment-api/catalog returns valid JSON array with expected fields (id, name, day_rate, category). 7. Health endpoint test: /health/ready returns 200 when DB and Valkey connected, 503 when either is down. 8. Cargo workspace builds all three service binaries from single `cargo build --workspace` command.

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.