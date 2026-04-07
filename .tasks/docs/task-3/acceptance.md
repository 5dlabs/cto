## Acceptance Criteria

- [ ] 1. POST /api/v1/opportunities with valid payload returns 201 with id field and status=pending. 2. POST /api/v1/opportunities/:id/convert returns 200 and creates a project row (verify via GET /api/v1/projects/:id). 3. ScoreLead returns GREEN for opportunity with verified customer, YELLOW for unvetted, RED for flagged — verified by seeding vetting results and calling POST /api/v1/opportunities/:id/score. 4. CheckOut creates inventory_transaction of type=checkout; second CheckOut for same item in same window returns 409 Conflict. 5. GET /health/ready returns 200 with postgres and redis reachable. 6. gRPC reflection accessible on port 9090 (grpcurl list returns service names). 7. GET /internal/gdpr/delete/:id anonymizes customer_id in opportunities and projects (re-query returns null customer_id). 8. go test ./... passes with >= 80% coverage.

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.