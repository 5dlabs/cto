Implement subtask 5003: Implement migration execution engine with batch processing and retry logic

## Objective
Create `src/modules/hermes/migration/artifact-migrator.ts` — the main migration orchestrator that processes legacy artifacts in configurable batches, copies binary data to the Hermes MinIO bucket via `IHermesArtifactWriter`, creates metadata records, and handles retries with exponential backoff.

## Steps
Step-by-step:
1. Implement `ArtifactMigrator` class that accepts `LegacyScanner`, `MigrationTracker`, and `IHermesArtifactWriter` as dependencies.
2. `migrate()` method orchestrates the full pipeline:
   a. Call scanner to get manifest.
   b. Split manifest into batches of configurable size (env var `MIGRATION_BATCH_SIZE`, default 50).
   c. For each batch, process artifacts sequentially within the batch:
      - Check tracker: skip if already migrated.
      - Stream binary data from legacy source to Hermes MinIO bucket under `legacy/{original_path}` using `IHermesArtifactWriter.writeArtifact()`.
      - Create artifact metadata record with `artifact_type: 'legacy_snapshot'`, `source: 'migration'`.
      - Mark as migrated in tracker.
   d. On failure for a single artifact: retry up to 3 times with exponential backoff (1s, 4s, 16s). If all retries fail, mark as failed in tracker and continue to next artifact.
3. Emit structured logs for each step: `migration_step: 'copy'` during copy, `migration_step: 'record'` during metadata creation.
4. Emit progress logs after each batch: `migration_progress: { total, completed, failed, skipped }`.
5. Return a `MigrationResult` summary object with final counts.
6. Ensure all writes go through `IHermesArtifactWriter` — no direct MinIO client calls in the migrator.

## Validation
Integration test with mock MinIO: Create 100 test artifacts in legacy storage, run migration, verify all 100 artifacts exist in the Hermes bucket at `legacy/` prefix with correct content. Verify tracker shows 100 completed, 0 failed. Retry test: mock `IHermesArtifactWriter.writeArtifact` to fail twice then succeed for specific artifacts — verify they still end up migrated after retries. Batch test: set batch size to 10, verify progress logs show 10 batches processed. Memory test: verify peak memory stays reasonable during migration of 100 artifacts (no full manifest loaded in memory at once during copy).