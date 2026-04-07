## Acceptance Criteria

- [ ] 1. GET /api/v1/catalog/products returns JSON array with at least 1 seeded product, status 200. 2. GET /api/v1/catalog/products/:id/availability?from=2025-01-01&to=2025-01-03 returns availability object with quantity_available >= 0 in under 500ms measured via curl -w '%{time_total}'. 3. POST /api/v1/catalog/products without JWT returns 401. 4. GET /health/ready returns 200 when postgres and valkey are reachable, 503 when either is down (test by temporarily scaling postgres to 0). 5. GET /metrics returns Prometheus text format with requests_total and request_duration_seconds metrics. 6. Rate limiter: 101 rapid requests from same IP returns HTTP 429 on the 101st. 7. GET /internal/gdpr/export/:id returns customer checkout JSON; subsequent DELETE /internal/gdpr/delete/:id causes re-fetch to return empty result. 8. cargo test passes with >= 80% coverage reported by cargo-tarpaulin.

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.