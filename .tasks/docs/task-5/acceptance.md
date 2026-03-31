## Acceptance Criteria

- [ ] 1. Idempotency: Running the migration twice on the same dataset results in the same number of artifact records — no duplicates (verified by COUNT query before and after second run).
- [ ] 2. Artifact integrity: For a sample of 10 migrated artifacts, the MD5 hash of the source file matches the MD5 hash of the object in the Hermes MinIO bucket.
- [ ] 3. Progress logging: During migration of 100+ artifacts, Loki query `{app="hermes"} | json | migration_step="copy"` returns log entries with incrementing `migration_progress.completed` values.
- [ ] 4. Failure handling: When MinIO is temporarily unavailable during migration, the migrator retries with backoff and resumes from the last successful artifact (verified by killing MinIO during migration, restarting it, and confirming migration completes).
- [ ] 5. Backward compatibility: After migration completes, existing legacy artifact access paths (original URLs/API endpoints) still return the correct artifacts.
- [ ] 6. Admin endpoint: `POST /api/hermes/admin/migrate-artifacts` without `hermes:admin` claim returns 403; with the claim, returns 202 with a migration job ID.

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.