Implement subtask 5002: Implement migration tracker with idempotency guarantees

## Objective
Create `src/modules/hermes/migration/migration-tracker.ts` that tracks per-artifact migration status, supports idempotent re-runs by recording which artifacts have been successfully migrated, and allows resuming from the last successful batch on restart.

## Steps
Step-by-step:
1. Define a `MigrationStatus` type: `{ artifactId: string, status: 'pending' | 'migrated' | 'failed', migratedAt?: Date, errorMessage?: string }`.
2. Implement `MigrationTracker` class with methods:
   - `isAlreadyMigrated(artifactId: string): Promise<boolean>` — checks if artifact was already migrated
   - `markMigrated(artifactId: string): Promise<void>` — records successful migration
   - `markFailed(artifactId: string, error: string): Promise<void>` — records failure
   - `getProgress(): Promise<{ total: number, completed: number, failed: number, skipped: number }>` — returns current progress
   - `resetFailed(): Promise<void>` — resets failed artifacts to pending for retry
3. Use a persistence backend (database table or JSON file — keep it behind an interface `IMigrationTrackerStore` so it can be swapped).
4. Ensure all state mutations are atomic — if the process crashes mid-write, the artifact is NOT marked as migrated.
5. On initialization, load existing state to support resume-from-where-we-left-off semantics.

## Validation
Unit test: Create tracker, mark 5 artifacts as migrated, verify `isAlreadyMigrated` returns true for those 5 and false for an unknown ID. Test idempotency: call `markMigrated` twice for the same artifact — no error, no duplicate records. Test resume: create tracker with 3 migrated and 2 failed, call `getProgress()`, assert counts are correct. Test `resetFailed`: after reset, previously failed artifacts return false from `isAlreadyMigrated`.