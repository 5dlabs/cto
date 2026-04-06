## Acceptance Criteria

- [ ] 1. POST /api/v1/social/upload with 3 test JPEG files returns 202 with draft_id and status=pending_curation. 2. After curation pipeline completes (poll GET /api/v1/social/drafts/:id), status=ready_for_approval and selected_photo_urls contains at least 1 URL pointing to R2 bucket. 3. POST /api/v1/social/drafts/:id/approve returns 200 and status transitions to approved. 4. POST /api/v1/social/drafts/:id/publish with approved draft returns 200; GET /api/v1/social/published includes the new post with platform=instagram and status=published. 5. Effect retry: mock Instagram API to fail twice then succeed; verify retry count = 2 in logs and final status=published. 6. GET /metrics returns prom-client text format with http_requests_total counter. 7. GET /health/ready returns 200 with all dependencies healthy. 8. vitest or jest test suite passes with >= 80% coverage including Effect service unit tests.

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.