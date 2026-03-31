## Acceptance Criteria

- [ ] 1. Integration test: Triggering a deliberation via `POST /api/hermes/deliberations` with a valid target URL results in at least one `current_site_screenshot` artifact record in the database within 30 seconds.
- [ ] 2. MinIO verification: After artifact generation completes, `aws s3 ls s3://hermes-artifacts-dev/{deliberation_id}/current-site/` (using MinIO endpoint) returns at least one PNG object with size > 0 bytes.
- [ ] 3. Presigned URL: `GET /api/hermes/artifacts/:id/url` returns a URL that, when fetched via HTTP GET, returns a 200 with `content-type: image/png`.
- [ ] 4. Variant snapshots: After deliberation completion, `GET /api/hermes/deliberations/:id/artifacts` returns both `current_site_screenshot` and `variant_snapshot` type artifacts.
- [ ] 5. Failure handling: When screenshot capture fails (invalid URL), the deliberation record status is `failed` and a structured log with `error_code: CAPTURE_FAILED` is emitted (queryable in Loki).
- [ ] 6. Object tags: MinIO objects created by the service have `retention-class=hermes` tag verified via `aws s3api get-object-tagging`.

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.