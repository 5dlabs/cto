Implement subtask 3004: Create database migrations for all 7 RMS schema tables

## Objective
Implement golang-migrate migration files for the `rms` schema including all 7 tables (opportunities, opportunity_line_items, projects, inventory_transactions, crew_members, crew_assignments, deliveries) with proper indexes, foreign keys, enums, and constraints.

## Steps
1. Install golang-migrate CLI and add `github.com/golang-migrate/migrate/v4` to go.mod.
2. Create migration `001_create_rms_schema.up.sql`: `CREATE SCHEMA IF NOT EXISTS rms;`
3. Create migration `002_create_opportunities.up.sql`:
   - Create enum types: `rms.opportunity_status` (pending, qualified, approved, converted), `rms.lead_score` (green, yellow, red)
   - `rms.opportunities` table with all columns per details, id as UUID with gen_random_uuid() default, timestamps with defaults
   - `rms.opportunity_line_items` table with FK to opportunities (ON DELETE CASCADE), subtotal_cents as computed or stored
   - Indexes: opportunities(status), opportunities(customer_id), opportunities(event_date_start), opportunity_line_items(opportunity_id)
4. Create migration `003_create_projects.up.sql`:
   - Create enum: `rms.project_status` (confirmed, in_progress, completed, cancelled)
   - `rms.projects` table with FK to opportunities (nullable, ON DELETE SET NULL), indexes on status, customer_id, opportunity_id
5. Create migration `004_create_inventory_transactions.up.sql`:
   - Create enum: `rms.transaction_type` (checkout, checkin, transfer)
   - `rms.inventory_transactions` table with nullable project_id FK, indexes on inventory_item_id, project_id, type, timestamp
6. Create migration `005_create_crew.up.sql`:
   - `rms.crew_members` table, `rms.crew_assignments` table with FKs to projects and crew_members
   - Indexes: crew_assignments(project_id), crew_assignments(crew_member_id), composite index on (crew_member_id, start_time, end_time) for conflict detection queries
7. Create migration `006_create_deliveries.up.sql`:
   - Create enum: `rms.delivery_status` (scheduled, in_transit, delivered, cancelled)
   - `rms.deliveries` table with FK to projects, indexes on project_id, status, scheduled_at
8. Create corresponding `.down.sql` files for each migration.
9. Create `internal/db/migrate.go` helper function that runs migrations on startup using the database URL from environment.

## Validation
Run all up migrations against a clean PostgreSQL database and verify all tables, enums, indexes, and foreign keys are created. Run all down migrations and verify clean rollback. Run up migrations a second time to ensure idempotency. Query pg_catalog to confirm all expected indexes exist.