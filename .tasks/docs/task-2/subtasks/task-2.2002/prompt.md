Implement subtask 2002: Create database migrations for deliberations and hermes_artifacts tables

## Objective
Write and validate database migration scripts that create the deliberations and hermes_artifacts tables in PostgreSQL without affecting existing tables.

## Steps
1. Create a new migration file following the project's existing migration convention (timestamp-prefixed SQL or migration tool format).
2. Create `deliberations` table:
   - `id` UUID PRIMARY KEY DEFAULT gen_random_uuid()
   - `status` VARCHAR(20) NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'processing', 'completed', 'failed'))
   - `input_payload` JSONB NOT NULL
   - `result_payload` JSONB
   - `triggered_by` UUID NOT NULL (FK to existing users table — verify FK column name)
   - `created_at` TIMESTAMPTZ NOT NULL DEFAULT now()
   - `updated_at` TIMESTAMPTZ NOT NULL DEFAULT now()
   - Index on `status` for filtering
   - Index on `triggered_by` for user-scoped queries
   - Index on `created_at` for pagination ordering
3. Create `hermes_artifacts` table:
   - `id` UUID PRIMARY KEY DEFAULT gen_random_uuid()
   - `deliberation_id` UUID NOT NULL REFERENCES deliberations(id) ON DELETE CASCADE
   - `name` VARCHAR(255) NOT NULL
   - `mime_type` VARCHAR(127) NOT NULL
   - `storage_path` TEXT NOT NULL
   - `size_bytes` BIGINT NOT NULL
   - `created_at` TIMESTAMPTZ NOT NULL DEFAULT now()
   - Index on `deliberation_id`
4. Create a down migration that drops both tables (hermes_artifacts first due to FK).
5. Test: run migration up on a clean database, then run on a database with existing legacy tables to confirm no interference.

## Validation
Migration runs successfully on a clean database — `\dt` shows `deliberations` and `hermes_artifacts` tables with correct columns and constraints. Migration runs on an existing database without altering any pre-existing tables. Down migration cleanly drops both tables.