Implement task 5: Preserve and Migrate Existing Snapshot Artifacts (Nova - Bun/Elysia)

## Goal
Build a migration pipeline that preserves existing legacy snapshot artifacts by copying them to the dedicated MinIO bucket and creating corresponding artifact metadata records, ensuring backward compatibility while enabling the Hermes path to access historical data.

## Task Context
- Agent owner: nova
- Stack: Bun/Elysia
- Priority: medium
- Dependencies: 1, 2, 3

## Implementation Plan
Step-by-step implementation:

1. **Migration module:** Create `src/modules/hermes/migration/` subdirectory:
   - `artifact-migrator.ts` — main migration orchestrator
   - `legacy-scanner.ts` — discovers existing snapshot artifacts from the legacy storage location
   - `migration-tracker.ts` — tracks migration progress per artifact (idempotent)

2. **Legacy artifact discovery:** Scan the legacy artifact storage (determine actual location — could be filesystem, existing MinIO bucket under GitLab, or database BLOBs):
   - Query existing artifact records/files to build a manifest of what needs migration
   - Log the total count and total size before beginning migration

3. **Migration execution:**
   - For each legacy artifact:
     a. Copy the binary data to the dedicated Hermes MinIO bucket under `legacy/{original_path}`
     b. Create an artifact metadata record via the `IHermesArtifactWriter` abstraction with `artifact_type: 'legacy_snapshot'` and `source: 'migration'`
     c. Mark the artifact as migrated in the tracker (prevent re-migration on retry)
   - Implement batch processing (configurable batch size, default 50) to avoid memory pressure
   - Implement retry logic with exponential backoff for failed copies

4. **Idempotency:** The migration must be safely re-runnable:
   - Check migration tracker before processing each artifact
   - Skip already-migrated artifacts
   - Resume from last successful batch on restart

5. **Abstraction layer compliance (D6 pending):** All artifact writes go through `IHermesArtifactWriter` — the migration code must not directly write to a specific table. When D6 is resolved, only the repository implementation changes.

6. **CLI/API trigger:** Expose migration as both:
   - CLI command: `bun run migrate:artifacts` for manual execution
   - Admin API endpoint: `POST /api/hermes/admin/migrate-artifacts` (requires admin-level RBAC claim `hermes:admin`)

7. **Progress reporting:** Emit structured logs during migration:
   - `migration_step: 'scan' | 'copy' | 'record' | 'verify' | 'complete'`
   - `migration_progress: { total: N, completed: M, failed: F, skipped: S }`
   - `error_code` for any failures

8. **Backward compatibility:** Existing legacy artifact access paths must continue to work. The migration copies data — it does NOT delete or modify the legacy storage location.

9. **Verification step:** After migration completes, verify:
   - All migrated artifacts are readable from the new MinIO bucket
   - Artifact count in new storage matches the migration manifest
   - Emit a final structured log with `migration_step: 'complete'` and full counts

## Acceptance Criteria
1. Idempotency: Running the migration twice on the same dataset results in the same number of artifact records — no duplicates (verified by COUNT query before and after second run).
2. Artifact integrity: For a sample of 10 migrated artifacts, the MD5 hash of the source file matches the MD5 hash of the object in the Hermes MinIO bucket.
3. Progress logging: During migration of 100+ artifacts, Loki query `{app="hermes"} | json | migration_step="copy"` returns log entries with incrementing `migration_progress.completed` values.
4. Failure handling: When MinIO is temporarily unavailable during migration, the migrator retries with backoff and resumes from the last successful artifact (verified by killing MinIO during migration, restarting it, and confirming migration completes).
5. Backward compatibility: After migration completes, existing legacy artifact access paths (original URLs/API endpoints) still return the correct artifacts.
6. Admin endpoint: `POST /api/hermes/admin/migrate-artifacts` without `hermes:admin` claim returns 403; with the claim, returns 202 with a migration job ID.

## Subtasks
- Implement legacy artifact scanner and manifest builder: Create `src/modules/hermes/migration/legacy-scanner.ts` that discovers existing snapshot artifacts from the legacy storage location, builds an in-memory manifest of all artifacts needing migration, and logs total count and total size. Exports a function that returns a structured manifest array with artifact identifiers, paths, and sizes.
- Implement migration tracker with idempotency guarantees: Create `src/modules/hermes/migration/migration-tracker.ts` that tracks per-artifact migration status, supports idempotent re-runs by recording which artifacts have been successfully migrated, and allows resuming from the last successful batch on restart.
- Implement migration execution engine with batch processing and retry logic: Create `src/modules/hermes/migration/artifact-migrator.ts` — the main migration orchestrator that processes legacy artifacts in configurable batches, copies binary data to the Hermes MinIO bucket via `IHermesArtifactWriter`, creates metadata records, and handles retries with exponential backoff.
- Implement CLI command for migration trigger: Create a CLI entry point `bun run migrate:artifacts` that invokes the ArtifactMigrator, handles process lifecycle (graceful shutdown), and reports final results to stdout.
- Implement admin API endpoint for migration trigger with RBAC: Create `POST /api/hermes/admin/migrate-artifacts` Elysia route that starts a migration job asynchronously, requires `hermes:admin` RBAC claim, and returns a 202 with a job ID for tracking.
- Implement post-migration verification step: Add a verification phase to the migration pipeline that validates all migrated artifacts are readable from the new MinIO bucket, confirms artifact counts match the manifest, computes integrity checksums, and emits the final structured completion log.

## Deliverables
- Update the relevant code, configuration, and tests.
- Keep artifacts aligned with the acceptance criteria.
- Document blockers or assumptions in your final summary.