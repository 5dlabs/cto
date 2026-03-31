Implement subtask 5004: Implement CLI command for migration trigger

## Objective
Create a CLI entry point `bun run migrate:artifacts` that invokes the ArtifactMigrator, handles process lifecycle (graceful shutdown), and reports final results to stdout.

## Steps
Step-by-step:
1. Create `src/modules/hermes/migration/cli.ts` (or add to existing CLI infrastructure).
2. Register a `migrate:artifacts` command in `package.json` scripts: `"migrate:artifacts": "bun run src/modules/hermes/migration/cli.ts"`.
3. The CLI script should:
   a. Initialize dependencies (MinIO client from ConfigMap, database connection, IHermesArtifactWriter, MigrationTracker, LegacyScanner).
   b. Call `migrator.migrate()` and await completion.
   c. Print final migration summary to stdout in a human-readable format.
   d. Exit with code 0 on success (even if some artifacts failed), exit code 1 on unrecoverable errors.
4. Handle SIGINT/SIGTERM gracefully: finish current artifact, save tracker state, then exit.
5. Accept optional CLI flags: `--batch-size=N`, `--dry-run` (scan only, no copies).

## Validation
Run `bun run migrate:artifacts --dry-run` against a test dataset — verify it logs the manifest summary without copying any artifacts. Run `bun run migrate:artifacts` against 10 test artifacts — verify exit code 0 and stdout contains migration summary with 10 completed. Test SIGINT handling: start migration of 50 artifacts, send SIGINT after ~5 complete, verify process exits cleanly and tracker shows ~5 migrated.