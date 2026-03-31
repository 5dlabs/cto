Implement subtask 5001: Implement legacy artifact scanner and manifest builder

## Objective
Create `src/modules/hermes/migration/legacy-scanner.ts` that discovers existing snapshot artifacts from the legacy storage location, builds an in-memory manifest of all artifacts needing migration, and logs total count and total size. Exports a function that returns a structured manifest array with artifact identifiers, paths, and sizes.

## Steps
Step-by-step:
1. Create the `src/modules/hermes/migration/` directory structure.
2. Define a `LegacyArtifactManifestEntry` type with fields: `id`, `originalPath`, `sizeBytes`, `lastModified`, `md5Hash` (if available from source).
3. Implement `scanLegacyArtifacts()` in `legacy-scanner.ts` that connects to the legacy storage location (filesystem/MinIO/DB — use an environment variable `LEGACY_ARTIFACT_SOURCE_TYPE` to switch between adapters).
4. For filesystem: recursively walk the configured directory. For MinIO: list objects in the legacy bucket. For DB: query the artifact table.
5. Return a `LegacyArtifactManifest` with the full list and summary stats (`totalCount`, `totalSizeBytes`).
6. Emit structured log with `migration_step: 'scan'` and `migration_progress: { total: N, completed: 0, failed: 0, skipped: 0 }`.
7. Export the manifest type and scanner function for use by the migration orchestrator.

## Validation
Seed a test legacy storage location (e.g., temp directory with 20 sample files of known sizes). Call `scanLegacyArtifacts()` and assert: returned manifest has exactly 20 entries, total size matches sum of file sizes, each entry has a valid `originalPath` and `sizeBytes`. Verify structured log output contains `migration_step: 'scan'` with correct total count.