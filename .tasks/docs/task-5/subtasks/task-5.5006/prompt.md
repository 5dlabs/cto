Implement subtask 5006: Implement post-migration verification step

## Objective
Add a verification phase to the migration pipeline that validates all migrated artifacts are readable from the new MinIO bucket, confirms artifact counts match the manifest, computes integrity checksums, and emits the final structured completion log.

## Steps
Step-by-step:
1. Add a `verify()` method to `ArtifactMigrator` (or create a separate `migration-verifier.ts`).
2. Verification steps:
   a. Query the migration tracker for all artifacts marked as migrated.
   b. For each migrated artifact, perform a HEAD request against the Hermes MinIO bucket to confirm the object exists and is readable.
   c. For a configurable sample size (default 10, env var `MIGRATION_VERIFY_SAMPLE_SIZE`), download the full object and compute MD5 hash, compare against the source artifact's hash.
   d. Compare total count of migrated artifacts in tracker against the original manifest total (minus any permanently failed).
3. Emit structured logs:
   - `migration_step: 'verify'` during verification
   - `migration_step: 'complete'` at the end with full counts: `{ total, completed, failed, skipped, verified, integrity_mismatches }`
4. If any integrity mismatches are found, log them with `error_code: 'INTEGRITY_MISMATCH'` and the affected artifact IDs.
5. Return a `VerificationResult` object that the CLI/API can include in final output.
6. Integrate: call `verify()` automatically at the end of `migrate()` unless `--skip-verify` flag is passed.

## Validation
Test happy path: Migrate 20 test artifacts, run verify — all 20 pass HEAD check, sample integrity checks pass, final log shows 0 mismatches. Test integrity mismatch: After migration, manually corrupt one object in MinIO, run verify with sample including that object — verify `INTEGRITY_MISMATCH` error log is emitted. Test count mismatch: Manually delete one object from MinIO after migration — verify the count mismatch is reported in the completion log. Verify the structured log `migration_step: 'complete'` contains all required count fields.